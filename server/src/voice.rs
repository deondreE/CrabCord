// src/voice.rs
// ─────────────────────────────────────────────────────────────────────────────
// Voice channel REST handlers + WebRTC signaling WebSocket broker.
//
// No audio ever touches this server. It only routes JSON signaling messages
// (SDP offers/answers, ICE candidates) between peers so they can establish
// direct P2P connections.
// ─────────────────────────────────────────────────────────────────────────────

use std::sync::Arc;

use axum::{
    Json,
    extract::{
        Path, State, WebSocketUpgrade,
        ws::{Message as WsMessage, WebSocket},
    },
    http::StatusCode,
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use uuid::Uuid;
use validator::Validate;

use crate::{
    extractor::AuthUser,
    models::{
        ClientSignal, CreateVoiceChannel, RoomPeer, ServerSignal, UpdateVoiceState, VoiceChannel,
        VoiceParticipant,
    },
    state::{AppState, VoiceRoom, VoiceSession},
};

// ── Shared DB helpers ─────────────────────────────────────────────────────────

async fn fetch_participants(
    db: &sqlx::PgPool,
    vc_id: Uuid,
) -> Result<Vec<VoiceParticipant>, (StatusCode, String)> {
    sqlx::query_as(
        r#"
        SELECT vp.voice_channel_id, vp.user_id, u.username, u.avatar_url,
               vp.muted, vp.deafened, vp.joined_at
        FROM voice_participants vp
        JOIN users u ON u.id = vp.user_id
        WHERE vp.voice_channel_id = $1
        ORDER BY vp.joined_at ASC
        "#,
    )
    .bind(vc_id)
    .fetch_all(db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// Broadcast a ServerSignal to every session in a room except `exclude_user`.
async fn broadcast_to_room(room: &VoiceRoom, signal: &ServerSignal, exclude_user: Option<Uuid>) {
    for session in room.iter() {
        if exclude_user == Some(session.user_id) {
            continue;
        }
        let _ = session.tx.send(signal.clone());
    }
}

// ── REST handlers ─────────────────────────────────────────────────────────────

/// POST /servers/:server_id/voice
/// Creates a voice channel. Only the server owner can do this.
pub async fn create_voice_channel(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<Uuid>,
    auth: AuthUser,
    Json(payload): Json<CreateVoiceChannel>,
) -> Result<Json<VoiceChannel>, (StatusCode, String)> {
    payload
        .validate()
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    if let Some(n) = payload.max_users {
        if n < 1 {
            return Err((StatusCode::BAD_REQUEST, "max_users must be ≥ 1".to_string()));
        }
    }

    let is_owner: Option<(Uuid,)> =
        sqlx::query_as("SELECT owner_id FROM servers WHERE id = $1 AND owner_id = $2")
            .bind(server_id)
            .bind(auth.0.sub)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if is_owner.is_none() {
        return Err((
            StatusCode::FORBIDDEN,
            "You are not the server owner".to_string(),
        ));
    }

    let vc: VoiceChannel = sqlx::query_as(
        r#"
        INSERT INTO voice_channels (server_id, name, max_users)
        VALUES ($1, $2, $3)
        RETURNING id, server_id, name, max_users, created_at
        "#,
    )
    .bind(server_id)
    .bind(&payload.name)
    .bind(payload.max_users)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::CONFLICT, e.to_string()))?;

    Ok(Json(vc))
}

/// GET /servers/:server_id/voice
/// Lists all voice channels with their current participant lists.
pub async fn get_voice_channels(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let channels: Vec<VoiceChannel> = sqlx::query_as(
        "SELECT id, server_id, name, max_users, created_at
         FROM voice_channels WHERE server_id = $1 ORDER BY created_at ASC",
    )
    .bind(server_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut out = Vec::with_capacity(channels.len());
    for vc in channels {
        let participants = fetch_participants(&state.db, vc.id).await?;
        out.push(serde_json::json!({
            "id":           vc.id,
            "server_id":    vc.server_id,
            "name":         vc.name,
            "max_users":    vc.max_users,
            "created_at":   vc.created_at,
            "participants": participants,
        }));
    }

    Ok(Json(serde_json::Value::Array(out)))
}

/// DELETE /servers/:server_id/voice/:vc_id
/// Deletes a voice channel. Only the server owner can do this.
/// All connected signaling sessions are evicted.
pub async fn delete_voice_channel(
    State(state): State<Arc<AppState>>,
    Path((server_id, vc_id)): Path<(Uuid, Uuid)>,
    auth: AuthUser,
) -> Result<StatusCode, (StatusCode, String)> {
    let is_owner: Option<(Uuid,)> =
        sqlx::query_as("SELECT owner_id FROM servers WHERE id = $1 AND owner_id = $2")
            .bind(server_id)
            .bind(auth.0.sub)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if is_owner.is_none() {
        return Err((
            StatusCode::FORBIDDEN,
            "You are not the server owner".to_string(),
        ));
    }

    let result = sqlx::query("DELETE FROM voice_channels WHERE id = $1 AND server_id = $2")
        .bind(vc_id)
        .bind(server_id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Voice channel not found".to_string()));
    }

    // Drop all sessions — their WS receivers will close, sending PeerLeft.
    state.voice_rooms.write().await.remove(&vc_id);

    Ok(StatusCode::NO_CONTENT)
}

/// POST /voice/:vc_id/join
/// Registers the user as a participant in the DB. Must be called before
/// opening the WebSocket stream so the room sees them as a legitimate peer.
pub async fn join_voice(
    State(state): State<Arc<AppState>>,
    Path(vc_id): Path<Uuid>,
    auth: AuthUser,
) -> Result<Json<VoiceParticipant>, (StatusCode, String)> {
    let vc: Option<VoiceChannel> = sqlx::query_as(
        "SELECT id, server_id, name, max_users, created_at FROM voice_channels WHERE id = $1",
    )
    .bind(vc_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let vc = vc.ok_or((StatusCode::NOT_FOUND, "Voice channel not found".to_string()))?;

    if let Some(max) = vc.max_users {
        let (count,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM voice_participants WHERE voice_channel_id = $1")
                .bind(vc_id)
                .fetch_one(&state.db)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        if count >= max as i64 {
            return Err((StatusCode::CONFLICT, "Voice channel is full".to_string()));
        }
    }

    // Upsert so reconnects are handled gracefully.
    let participant: VoiceParticipant = sqlx::query_as(
        r#"
        INSERT INTO voice_participants (voice_channel_id, user_id)
        VALUES ($1, $2)
        ON CONFLICT (voice_channel_id, user_id) DO UPDATE SET joined_at = now()
        RETURNING
            voice_channel_id,
            user_id,
            (SELECT username   FROM users WHERE id = $2) AS username,
            (SELECT avatar_url FROM users WHERE id = $2) AS avatar_url,
            muted,
            deafened,
            joined_at
        "#,
    )
    .bind(vc_id)
    .bind(auth.0.sub)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(participant))
}

/// POST /voice/:vc_id/leave
/// Removes the user from the DB participant list and evicts their signaling session.
pub async fn leave_voice(
    State(state): State<Arc<AppState>>,
    Path(vc_id): Path<Uuid>,
    auth: AuthUser,
) -> Result<StatusCode, (StatusCode, String)> {
    let result =
        sqlx::query("DELETE FROM voice_participants WHERE voice_channel_id = $1 AND user_id = $2")
            .bind(vc_id)
            .bind(auth.0.sub)
            .execute(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            "You are not in this voice channel".to_string(),
        ));
    }

    // Evict the in-memory session — the WS loop will notice and send PeerLeft.
    let mut rooms = state.voice_rooms.write().await;
    if let Some(room) = rooms.get_mut(&vc_id) {
        room.retain(|s| s.user_id != auth.0.sub);
        if room.is_empty() {
            rooms.remove(&vc_id);
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

/// PATCH /voice/:vc_id/state
/// Updates the user's mute/deafen state. Returns the updated participant row.
pub async fn update_voice_state(
    State(state): State<Arc<AppState>>,
    Path(vc_id): Path<Uuid>,
    auth: AuthUser,
    Json(payload): Json<UpdateVoiceState>,
) -> Result<Json<VoiceParticipant>, (StatusCode, String)> {
    let participant: Option<VoiceParticipant> = sqlx::query_as(
        r#"
        UPDATE voice_participants SET
            muted    = COALESCE($1, muted),
            deafened = COALESCE($2, deafened)
        WHERE voice_channel_id = $3 AND user_id = $4
        RETURNING
            voice_channel_id,
            user_id,
            (SELECT username   FROM users WHERE id = $4) AS username,
            (SELECT avatar_url FROM users WHERE id = $4) AS avatar_url,
            muted,
            deafened,
            joined_at
        "#,
    )
    .bind(payload.muted)
    .bind(payload.defened)
    .bind(vc_id)
    .bind(auth.0.sub)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    participant.map(Json).ok_or((
        StatusCode::NOT_FOUND,
        "You are not in this voice channel".to_string(),
    ))
}

/// GET /voice/:vc_id/participants
/// Returns all current participants with their mute/deafen state.
pub async fn get_voice_participants(
    State(state): State<Arc<AppState>>,
    Path(vc_id): Path<Uuid>,
    _auth: AuthUser,
) -> Result<Json<Vec<VoiceParticipant>>, (StatusCode, String)> {
    fetch_participants(&state.db, vc_id).await.map(Json)
}

/// GET /voice/:vc_id/stream  (WebSocket upgrade)
pub async fn voice_stream(
    State(state): State<Arc<AppState>>,
    Path(vc_id): Path<Uuid>,
    auth: AuthUser,
    ws: WebSocketUpgrade,
) -> Result<Response, (StatusCode, String)> {
    // Must have joined via REST first.
    let row: Option<(String, bool, bool)> = sqlx::query_as(
        r#"
        SELECT u.username, vp.muted, vp.deafened
        FROM voice_participants vp
        JOIN users u ON u.id = vp.user_id
        WHERE vp.voice_channel_id = $1 AND vp.user_id = $2
        "#,
    )
    .bind(vc_id)
    .bind(auth.0.sub)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let (username, _muted, _deafened) = row.ok_or((
        StatusCode::FORBIDDEN,
        "Call POST /voice/:vc_id/join before connecting the stream".to_string(),
    ))?;

    let user_id = auth.0.sub;

    Ok(ws.on_upgrade(move |socket| signaling_loop(socket, state, vc_id, user_id, username)))
}

// ── Core signaling loop ───────────────────────────────────────────────────────

async fn signaling_loop(
    socket: WebSocket,
    state: Arc<AppState>,
    vc_id: Uuid,
    user_id: Uuid,
    username: String,
) {
    let (mut ws_tx, mut ws_rx) = socket.split();

    // Channel for the server to push signals down to this client's WS sender.
    let (sig_tx, mut sig_rx) = mpsc::channel::<ServerSignal>(64);

    // ── Register session and snapshot existing peers ──────────────────────────
    let peers_snapshot: Vec<RoomPeer> = {
        let mut rooms = state.voice_rooms.write().await;
        let room = rooms.entry(vc_id).or_default();

        // Collect current peers for RoomState before inserting self.
        let snapshot = room
            .iter()
            .map(|s| {
                RoomPeer {
                    user_id: s.user_id,
                    username: s.username.clone(),
                    muted: false, // will be populated from DB snapshot below
                    deafened: false,
                }
            })
            .collect();

        room.push(Arc::new(VoiceSession {
            user_id,
            username: username.clone(),
            tx: sig_tx,
        }));

        snapshot
    };

    // Enrich snapshot with live muted/deafened state from DB.
    let room_state = {
        let db_participants = fetch_participants(&state.db, vc_id)
            .await
            .unwrap_or_default();

        let peers: Vec<RoomPeer> = peers_snapshot
            .into_iter()
            .map(|p| {
                let db = db_participants.iter().find(|r| r.user_id == p.user_id);
                RoomPeer {
                    user_id: p.user_id,
                    username: p.username,
                    muted: db.map(|r| r.muted).unwrap_or(false),
                    deafened: db.map(|r| r.deafened).unwrap_or(false),
                }
            })
            .collect();

        ServerSignal::RoomState { peers }
    };

    // Send RoomState to the newly connected client.
    if let Ok(json) = serde_json::to_string(&room_state) {
        let _ = ws_tx.send(WsMessage::Text(json)).await;
    }

    // Announce arrival to everyone else.
    {
        let rooms = state.voice_rooms.read().await;
        if let Some(room) = rooms.get(&vc_id) {
            let signal = ServerSignal::PeerJoined {
                user_id,
                username: username.clone(),
            };
            broadcast_to_room(room, &signal, Some(user_id)).await;
        }
    }

    // ── Main loop: two concurrent tasks ──────────────────────────────────────

    // Task A: drain sig_rx → send JSON down the WebSocket.
    let send_task = tokio::spawn(async move {
        while let Some(signal) = sig_rx.recv().await {
            match serde_json::to_string(&signal) {
                Ok(json) => {
                    if ws_tx.send(WsMessage::Text(json)).await.is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    // Task B: receive messages from the WebSocket → route to the target peer.
    loop {
        tokio::select! {
            msg = ws_rx.next() => {
                match msg {
                    Some(Ok(WsMessage::Text(raw))) => {
                        match serde_json::from_str::<ClientSignal>(&raw) {
                            Ok(signal) => {
                                handle_client_signal(
                                    &state, vc_id, user_id, &username, signal,
                                ).await;
                            }
                            Err(_) => {
                                // Ignore malformed messages — no need to disconnect.
                            }
                        }
                    }
                    Some(Ok(WsMessage::Close(_))) | None => break,
                    // Ping/pong handled automatically by axum.
                    _ => {}
                }
            }
        }
    }

    // ── Cleanup on disconnect ─────────────────────────────────────────────────
    send_task.abort();

    // Remove from in-memory room.
    {
        let mut rooms = state.voice_rooms.write().await;
        if let Some(room) = rooms.get_mut(&vc_id) {
            room.retain(|s| s.user_id != user_id);
            if room.is_empty() {
                rooms.remove(&vc_id);
            }
        }
    }

    // Announce departure to remaining peers.
    {
        let rooms = state.voice_rooms.read().await;
        if let Some(room) = rooms.get(&vc_id) {
            let signal = ServerSignal::PeerLeft { user_id };
            broadcast_to_room(room, &signal, None).await;
        }
    }
}

async fn handle_client_signal(
    state: &AppState,
    vc_id: Uuid,
    from: Uuid,
    username: &str,
    signal: ClientSignal,
) {
    let rooms = state.voice_rooms.read().await;
    let room = match rooms.get(&vc_id) {
        Some(r) => r,
        None => return,
    };

    match signal {
        // ── SDP offer: route to a specific peer ──────────────────────────────
        ClientSignal::Offer { to, sdp } => {
            if let Some(target) = room.iter().find(|s| s.user_id == to) {
                let _ = target.tx.send(ServerSignal::Offer { from, sdp });
            }
        }

        // ── SDP answer: route to a specific peer ─────────────────────────────
        ClientSignal::Answer { to, sdp } => {
            if let Some(target) = room.iter().find(|s| s.user_id == to) {
                let _ = target.tx.send(ServerSignal::Answer { from, sdp });
            }
        }

        // ── ICE candidate: route to a specific peer ──────────────────────────
        ClientSignal::IceCanidate { to, candidate } => {
            if let Some(target) = room.iter().find(|s| s.user_id == to) {
                let _ = target
                    .tx
                    .send(ServerSignal::IceCandidate { from, candidate });
            }
        }

        // ── Reconnect: re-announce to all peers so they re-initiate offers ───
        ClientSignal::Reconnecting => {
            let signal = ServerSignal::Reconnect { user_id: from };
            broadcast_to_room(room, &signal, Some(from)).await;

            // Also send a fresh RoomState back to the reconnecting client.
            drop(rooms); // release read lock before acquiring write
            let rooms = state.voice_rooms.read().await;
            if let Some(room) = rooms.get(&vc_id) {
                if let Some(me) = room.iter().find(|s| s.user_id == from) {
                    let peers: Vec<RoomPeer> = room
                        .iter()
                        .filter(|s| s.user_id != from)
                        .map(|s| RoomPeer {
                            user_id: s.user_id,
                            username: s.username.clone(),
                            muted: false,
                            deafened: false,
                        })
                        .collect();
                    let _ = me.tx.send(ServerSignal::RoomState { peers });
                }
            }
        }
    }
}
