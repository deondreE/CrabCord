mod auth;
mod extractor;
mod helpers;
mod models;
mod state;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, patch, post},
};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::{collections::HashMap, env, sync::Arc};
use tokio::sync::{RwLock, broadcast};
use validator::Validate;

use crate::{
    auth::{create_refresh_token, create_token, hash_password, refresh_expiry, verify_password},
    extractor::AuthUser,
    helpers::require_permission,
    models::{
        AssignRole, AuthResponse, Channel, CreateChannel, CreateDirectMessage, CreateInvite,
        CreateMessage, CreateRole, CreateServer, CreateUser, DirectMessage, Invite, LoginRequest,
        Message, RefreshRequest, Role, Server, ServerMember, UpdateChannel, UpdateDirectMessage,
        UpdateMessage, UpdateProfile, UpdateServer, UpdateStatus, User, UserRole, UserSearchQuery,
        permissions,
    },
    state::AppState,
};

use uuid::Uuid;

fn validate<T: Validate>(payload: &T) -> Result<(), (StatusCode, String)> {
    payload.validate().map_err(|e| {
        let messages: Vec<String> = e
            .field_errors()
            .into_iter()
            .flat_map(|(_, errors)| {
                errors
                    .iter()
                    .map(|err| err.message.clone().unwrap_or_default().to_string())
            })
            .collect();
        (StatusCode::BAD_REQUEST, messages.join(", "))
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            username TEXT NOT NULL UNIQUE,
            email TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            avatar_url TEXT,
            status TEXT NOT NULL DEFAULT 'online',
            created_at TIMESTAMPTZ NOT NULL DEFAULT now()
        );
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS servers (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name TEXT NOT NULL,
            owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            created_at TIMESTAMPTZ NOT NULL DEFAULT now()
        );
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS channels (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            server_id UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
            UNIQUE (server_id, name)
        );
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS server_members (
            server_id UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
            user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            joined_at TIMESTAMPTZ NOT NULL DEFAULT now(),
            PRIMARY KEY (server_id, user_id)
        );
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS messages (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            channel_id UUID NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
            user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            content TEXT NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
            edited_at TIMESTAMPTZ
        );
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS invites (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            code TEXT NOT NULL UNIQUE,
            server_id UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
            created_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            max_uses INT,
            uses INT NOT NULL DEFAULT 0,
            expires_at TIMESTAMPTZ,
            created_at TIMESTAMPTZ NOT NULL DEFAULT now()
        );
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS roles (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            server_id UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            permissions BIGINT NOT NULL DEFAULT 0,
            created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
            UNIQUE (server_id, name)
        );
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS user_roles (
            server_id UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
            user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
            assigned_at TIMESTAMPTZ NOT NULL DEFAULT now(),
            PRIMARY KEY (server_id, user_id, role_id)
        );
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS refresh_tokens (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            token TEXT NOT NULL UNIQUE,
            expires_at TIMESTAMPTZ NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT now()
        );
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS direct_messages (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            sender_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            receiver_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            content TEXT NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
            edited_at TIMESTAMPTZ,
            read_at TIMESTAMPTZ
        );
        "#,
    )
    .execute(&pool)
    .await?;

    let (tx, _rx) = broadcast::channel::<Message>(100);

    let shared_state = Arc::new(AppState {
        db: pool,
        tx,
        presence: RwLock::new(HashMap::new()),
    });

    let app = Router::new()
        .route("/auth/register", post(_register_handle))
        .route("/auth/login", post(_login_handle))
        .route("/auth/refresh", post(_refresh_handle))
        .route("/users/me", get(_get_me_handle))
        .route("/users/me", patch(_update_me_handle))
        .route("/users/me/avatar", post(_upload_avatar_handle))
        .route("/users/me/status", patch(_update_status_handle))
        .route("/users/search", get(_search_users_handle))
        .route("/users/:user_id/status", get(_get_user_status_handle))
        .route("/avatars/:filename", get(_get_avatar_handle))
        .route("/dm", get(_get_dm_conversations_handle))
        .route("/dm/:user_id", post(_send_dm_handle))
        .route("/dm/:user_id", get(_get_dm_handle))
        .route("/dm/:user_id/:message_id", patch(_edit_dm_handle))
        .route("/dm/:user_id/:message_id", delete(_delete_dm_handle))
        .route("/dm/:user_id/:message_id/read", post(_mark_dm_read_handle))
        .route("/servers", post(_create_server_handle))
        .route("/servers/:server_id", patch(_update_server_handle))
        .route("/servers/:server_id", delete(_delete_server_handle))
        .route("/servers/:server_id/join", post(_join_server_handle))
        .route("/servers/:server_id/leave", post(_leave_server_handle))
        .route("/servers/:server_id/members", get(_get_members_handle))
        .route(
            "/servers/:server_id/members/:user_id/roles",
            get(_get_user_roles_handle),
        )
        .route("/servers/:server_id/channels", post(_create_channel_handle))
        .route("/servers/:server_id/channels", get(_get_channels_handle))
        .route(
            "/servers/:server_id/channels/:channel_id",
            patch(_update_channel_handle),
        )
        .route(
            "/servers/:server_id/channels/:channel_id",
            delete(_delete_channel_handle),
        )
        .route("/servers/:server_id/roles", post(_create_role_handle))
        .route("/servers/:server_id/roles", get(_get_roles_handle))
        .route(
            "/servers/:server_id/roles/assign",
            post(_assign_role_handle),
        )
        .route(
            "/servers/:server_id/roles/revoke",
            post(_revoke_role_handle),
        )
        .route("/servers/:server_id/invites", post(_create_invite_handle))
        .route("/invites/:code", get(_get_invite_handle))
        .route("/invites/:code/join", post(_use_invite_handle))
        .route(
            "/channels/:channel_id/messages",
            post(_send_messages_handle),
        )
        .route("/channels/:channel_id/messages", get(_get_messages_handle))
        .route(
            "/channels/:channel_id/messages/:message_id",
            patch(_edit_message_handle),
        )
        .route(
            "/channels/:channel_id/messages/:message_id",
            delete(_delete_message_handle),
        )
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("🚀 Server active at http://localhost:3000");
    axum::serve(listener, app).await?;

    Ok(())
}

/// Creates a new message in the specified channel and broadcasts it.
async fn _send_messages_handle(
    State(state): State<Arc<AppState>>,
    Path(channel_id): Path<Uuid>,
    auth: AuthUser,
    Json(payload): Json<CreateMessage>,
) -> Result<Json<Message>, (StatusCode, String)> {
    validate(&payload)?;

    let channel_exists: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM channels WHERE id = $1")
        .bind(channel_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if channel_exists.is_none() {
        return Err((StatusCode::NOT_FOUND, "Channel not found".to_string()));
    }

    let message: Message = sqlx::query_as(
        r#"
        INSERT INTO messages (channel_id, user_id, content)
        VALUES ($1, $2, $3)
        RETURNING id, channel_id, user_id,
            (SELECT username FROM users WHERE id = messages.user_id) AS username,
            content, created_at, edited_at
        "#,
    )
    .bind(channel_id)
    .bind(auth.0.sub)
    .bind(payload.content)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let _ = state.tx.send(message.clone());
    Ok(Json(message))
}

/// Retrieves the last 50 messages from a channel including author usernames.
async fn _get_messages_handle(
    State(state): State<Arc<AppState>>,
    Path(channel_id): Path<Uuid>,
) -> Result<Json<Vec<Message>>, (StatusCode, String)> {
    let messages: Vec<Message> = sqlx::query_as(
        r#"
        SELECT m.id, m.channel_id, m.user_id, u.username, m.content, m.created_at, m.edited_at
        FROM messages m
        JOIN users u ON m.user_id = u.id
        WHERE m.channel_id = $1
        ORDER BY m.created_at ASC LIMIT 50
        "#,
    )
    .bind(channel_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(messages))
}

/// Edits the content of a message. Only the original author can edit.
async fn _edit_message_handle(
    State(state): State<Arc<AppState>>,
    Path((channel_id, message_id)): Path<(Uuid, Uuid)>,
    auth: AuthUser,
    Json(payload): Json<UpdateMessage>,
) -> Result<Json<Message>, (StatusCode, String)> {
    validate(&payload)?;

    let message: Option<Message> = sqlx::query_as(
        r#"
        UPDATE messages SET content = $1, edited_at = now()
        WHERE id = $2 AND channel_id = $3 AND user_id = $4
        RETURNING id, channel_id, user_id,
            (SELECT username FROM users WHERE id = messages.user_id) AS username,
            content, created_at, edited_at
        "#,
    )
    .bind(payload.content)
    .bind(message_id)
    .bind(channel_id)
    .bind(auth.0.sub)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    message.map(Json).ok_or((
        StatusCode::FORBIDDEN,
        "Message not found or you are not the author".to_string(),
    ))
}

/// Deletes a message. Only the original author can delete.
async fn _delete_message_handle(
    State(state): State<Arc<AppState>>,
    Path((channel_id, message_id)): Path<(Uuid, Uuid)>,
    auth: AuthUser,
) -> Result<StatusCode, (StatusCode, String)> {
    let result =
        sqlx::query("DELETE FROM messages WHERE id = $1 AND channel_id = $2 AND user_id = $3")
            .bind(message_id)
            .bind(channel_id)
            .bind(auth.0.sub)
            .execute(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::FORBIDDEN,
            "Message not found or you are not the author".to_string(),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Sends a direct message to another user.
async fn _send_dm_handle(
    State(state): State<Arc<AppState>>,
    Path(receiver_id): Path<Uuid>,
    auth: AuthUser,
    Json(payload): Json<CreateDirectMessage>,
) -> Result<Json<DirectMessage>, (StatusCode, String)> {
    validate(&payload)?;

    if auth.0.sub == receiver_id {
        return Err((
            StatusCode::BAD_REQUEST,
            "Cannot send DM to yourself".to_string(),
        ));
    }

    let receiver_exists: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM users WHERE id = $1")
        .bind(receiver_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if receiver_exists.is_none() {
        return Err((StatusCode::NOT_FOUND, "User not found".to_string()));
    }

    let dm: DirectMessage = sqlx::query_as(
        r#"
        INSERT INTO direct_messages (sender_id, receiver_id, content)
        VALUES ($1, $2, $3)
        RETURNING id, sender_id, receiver_id, content, created_at, edited_at, read_at
        "#,
    )
    .bind(auth.0.sub)
    .bind(receiver_id)
    .bind(payload.content)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(dm))
}

/// Retrieves DM history between the current user and another user.
async fn _get_dm_handle(
    State(state): State<Arc<AppState>>,
    Path(other_user_id): Path<Uuid>,
    auth: AuthUser,
) -> Result<Json<Vec<DirectMessage>>, (StatusCode, String)> {
    let messages: Vec<DirectMessage> = sqlx::query_as(
        r#"
        SELECT id, sender_id, receiver_id, content, created_at, edited_at, read_at
        FROM direct_messages
        WHERE (sender_id = $1 AND receiver_id = $2)
           OR (sender_id = $2 AND receiver_id = $1)
        ORDER BY created_at ASC
        LIMIT 50
        "#,
    )
    .bind(auth.0.sub)
    .bind(other_user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(messages))
}

/// Edits a direct message. Only the sender can edit.
async fn _edit_dm_handle(
    State(state): State<Arc<AppState>>,
    Path((_other_user_id, message_id)): Path<(Uuid, Uuid)>,
    auth: AuthUser,
    Json(payload): Json<UpdateDirectMessage>,
) -> Result<Json<DirectMessage>, (StatusCode, String)> {
    validate(&payload)?;

    let dm: Option<DirectMessage> = sqlx::query_as(
        r#"
        UPDATE direct_messages SET content = $1, edited_at = now()
        WHERE id = $2 AND sender_id = $3
        RETURNING id, sender_id, receiver_id, content, created_at, edited_at, read_at
        "#,
    )
    .bind(payload.content)
    .bind(message_id)
    .bind(auth.0.sub)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    dm.map(Json).ok_or((
        StatusCode::FORBIDDEN,
        "Message not found or you are not the sender".to_string(),
    ))
}

/// Deletes a direct message. Only the sender can delete.
async fn _delete_dm_handle(
    State(state): State<Arc<AppState>>,
    Path((_other_user_id, message_id)): Path<(Uuid, Uuid)>,
    auth: AuthUser,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = sqlx::query("DELETE FROM direct_messages WHERE id = $1 AND sender_id = $2")
        .bind(message_id)
        .bind(auth.0.sub)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::FORBIDDEN,
            "Message not found or you are not the sender".to_string(),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Marks a direct message as read. Only the receiver can mark as read.
async fn _mark_dm_read_handle(
    State(state): State<Arc<AppState>>,
    Path((_sender_id, message_id)): Path<(Uuid, Uuid)>,
    auth: AuthUser,
) -> Result<Json<DirectMessage>, (StatusCode, String)> {
    let dm: Option<DirectMessage> = sqlx::query_as(
        r#"
        UPDATE direct_messages SET read_at = now()
        WHERE id = $1 AND receiver_id = $2
        RETURNING id, sender_id, receiver_id, content, created_at, edited_at, read_at
        "#,
    )
    .bind(message_id)
    .bind(auth.0.sub)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    dm.map(Json).ok_or((
        StatusCode::NOT_FOUND,
        "Message not found or already read".to_string(),
    ))
}

/// Returns the latest message per DM conversation for the current user.
async fn _get_dm_conversations_handle(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<Vec<DirectMessage>>, (StatusCode, String)> {
    let conversations: Vec<DirectMessage> = sqlx::query_as(
        r#"
        SELECT DISTINCT ON (
            LEAST(sender_id, receiver_id),
            GREATEST(sender_id, receiver_id)
        )
        id, sender_id, receiver_id, content, created_at, edited_at, read_at
        FROM direct_messages
        WHERE sender_id = $1 OR receiver_id = $1
        ORDER BY
            LEAST(sender_id, receiver_id),
            GREATEST(sender_id, receiver_id),
            created_at DESC
        "#,
    )
    .bind(auth.0.sub)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(conversations))
}

/// Registers a new user and returns a JWT and refresh token.
async fn _register_handle(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    validate(&payload)?;

    let password_hash = hash_password(&payload.password)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let user: User = sqlx::query_as(
        r#"
        INSERT INTO users (username, email, password_hash)
        VALUES ($1, $2, $3)
        RETURNING id, username, email, avatar_url, status, created_at
        "#,
    )
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(&password_hash)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::CONFLICT, e.to_string()))?;

    let token = create_token(user.id, &user.username)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let refresh_token = create_refresh_token();
    let expires_at = refresh_expiry();

    sqlx::query("INSERT INTO refresh_tokens (user_id, token, expires_at) VALUES ($1, $2, $3)")
        .bind(user.id)
        .bind(&refresh_token)
        .bind(expires_at)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(AuthResponse {
        token,
        refresh_token,
        user,
    }))
}

/// Authenticates a user and returns a JWT and refresh token.
async fn _login_handle(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    let row: Option<(Uuid, String, String, String, Option<String>, chrono::DateTime<chrono::Utc>)> =
        sqlx::query_as(
            "SELECT id, username, password_hash, status, avatar_url, created_at FROM users WHERE email = $1",
        )
        .bind(&payload.email)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let (id, username, password_hash, status, avatar_url, created_at) = row.ok_or((
        StatusCode::UNAUTHORIZED,
        "Invalid email or password".to_string(),
    ))?;

    let valid = verify_password(&payload.password, &password_hash)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Invalid email or password".to_string(),
        ));
    }

    let user = User {
        id,
        username: username.clone(),
        email: payload.email,
        avatar_url,
        status,
        created_at,
    };

    let token = create_token(id, &username)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let refresh_token = create_refresh_token();
    let expires_at = refresh_expiry();

    sqlx::query("INSERT INTO refresh_tokens (user_id, token, expires_at) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(&refresh_token)
        .bind(expires_at)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(AuthResponse {
        token,
        refresh_token,
        user,
    }))
}

/// Issues a new JWT and rotates the refresh token.
async fn _refresh_handle(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RefreshRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    let row: Option<(Uuid, chrono::DateTime<chrono::Utc>)> =
        sqlx::query_as("SELECT user_id, expires_at FROM refresh_tokens WHERE token = $1")
            .bind(&payload.refresh_token)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let (user_id, expires_at) = row.ok_or((
        StatusCode::UNAUTHORIZED,
        "Invalid refresh token".to_string(),
    ))?;

    if chrono::Utc::now() > expires_at {
        sqlx::query("DELETE FROM refresh_tokens WHERE token = $1")
            .bind(&payload.refresh_token)
            .execute(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        return Err((
            StatusCode::UNAUTHORIZED,
            "Refresh token has expired".to_string(),
        ));
    }

    let user: User = sqlx::query_as(
        "SELECT id, username, email, avatar_url, status, created_at FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sqlx::query("DELETE FROM refresh_tokens WHERE token = $1")
        .bind(&payload.refresh_token)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let new_refresh_token = create_refresh_token();
    let new_expires_at = refresh_expiry();

    sqlx::query("INSERT INTO refresh_tokens (user_id, token, expires_at) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(&new_refresh_token)
        .bind(new_expires_at)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let token = create_token(user.id, &user.username)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(AuthResponse {
        token,
        refresh_token: new_refresh_token,
        user,
    }))
}

/// Returns the currently authenticated user.
async fn _get_me_handle(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<User>, (StatusCode, String)> {
    let user: Option<User> = sqlx::query_as(
        "SELECT id, username, email, avatar_url, status, created_at FROM users WHERE id = $1",
    )
    .bind(auth.0.sub)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    user.map(Json)
        .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))
}

/// Updates the current user's username, email, or password.
async fn _update_me_handle(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(payload): Json<UpdateProfile>,
) -> Result<Json<User>, (StatusCode, String)> {
    validate(&payload)?;

    let password_hash = match &payload.password {
        Some(pw) => Some(
            hash_password(pw).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
        ),
        None => None,
    };

    let user: Option<User> = sqlx::query_as(
        r#"
        UPDATE users SET
            username      = COALESCE($1, username),
            email         = COALESCE($2, email),
            password_hash = COALESCE($3, password_hash)
        WHERE id = $4
        RETURNING id, username, email, avatar_url, status, created_at
        "#,
    )
    .bind(payload.username)
    .bind(payload.email)
    .bind(password_hash)
    .bind(auth.0.sub)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    user.map(Json)
        .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))
}

/// Searches users by username using a case-insensitive partial match.
async fn _search_users_handle(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    axum::extract::Query(query): axum::extract::Query<UserSearchQuery>,
) -> Result<Json<Vec<User>>, (StatusCode, String)> {
    let users: Vec<User> = sqlx::query_as(
        r#"
        SELECT id, username, email, avatar_url, status, created_at
        FROM users
        WHERE username ILIKE $1
        AND id != $2
        LIMIT 20
        "#,
    )
    .bind(format!("%{}%", query.username))
    .bind(auth.0.sub)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(users))
}

/// Updates the current user's presence status.
async fn _update_status_handle(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(payload): Json<UpdateStatus>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let valid = ["online", "idle", "dnd", "offline"];
    if !valid.contains(&payload.status.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid status. Must be online, idle, dnd, or offline".to_string(),
        ));
    }

    {
        let mut presence = state.presence.write().await;
        presence.insert(auth.0.sub, payload.status.clone());
    }

    sqlx::query("UPDATE users SET status = $1 WHERE id = $2")
        .bind(&payload.status)
        .bind(auth.0.sub)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({ "status": payload.status })))
}

/// Returns a user's presence status, checking in-memory map first then falling back to DB.
async fn _get_user_status_handle(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    _auth: AuthUser,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let status = {
        let presence = state.presence.read().await;
        presence.get(&user_id).cloned()
    };

    let status = match status {
        Some(s) => s,
        None => {
            let row: Option<(String,)> = sqlx::query_as("SELECT status FROM users WHERE id = $1")
                .bind(user_id)
                .fetch_optional(&state.db)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            row.ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?
                .0
        }
    };

    Ok(Json(
        serde_json::json!({ "user_id": user_id, "status": status }),
    ))
}

/// Uploads a profile picture for the current user. Expects multipart field named "avatar".
async fn _upload_avatar_handle(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    mut multipart: axum::extract::Multipart,
) -> Result<Json<User>, (StatusCode, String)> {
    let avatars_dir = "avatars";

    tokio::fs::create_dir_all(avatars_dir)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut avatar_url: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    {
        let name = field.name().unwrap_or_default();
        if name != "avatar" {
            continue;
        }

        let file_name = format!("{}.png", auth.0.sub);
        let path = std::path::Path::new(avatars_dir).join(&file_name);

        let data = field
            .bytes()
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

        if data.len() > 5 * 1024 * 1024 {
            return Err((
                StatusCode::BAD_REQUEST,
                "Avatar must be under 5MB".to_string(),
            ));
        }

        tokio::fs::write(&path, &data)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        avatar_url = Some(format!("/avatars/{}", file_name));
        break;
    }

    let url = avatar_url.ok_or((
        StatusCode::BAD_REQUEST,
        "No file uploaded or field 'avatar' missing".to_string(),
    ))?;

    let user: User = sqlx::query_as(
        r#"
        UPDATE users SET avatar_url = $1
        WHERE id = $2
        RETURNING id, username, email, avatar_url, status, created_at
        "#,
    )
    .bind(&url)
    .bind(auth.0.sub)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(user))
}

/// Serves an avatar image file from disk.
async fn _get_avatar_handle(
    Path(filename): Path<String>,
) -> Result<axum::response::Response, (StatusCode, String)> {
    let path = format!("avatars/{}", filename);
    let data = std::fs::read(&path)
        .map_err(|_| (StatusCode::NOT_FOUND, "Avatar not found".to_string()))?;

    Ok(axum::response::Response::builder()
        .header("Content-Type", "image/png")
        .body(axum::body::Body::from(data))
        .unwrap())
}

/// Creates a new server and automatically adds the creator as the owner and first member.
async fn _create_server_handle(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(payload): Json<CreateServer>,
) -> Result<Json<Server>, (StatusCode, String)> {
    validate(&payload)?;

    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let server: Server = sqlx::query_as(
        r#"
        INSERT INTO servers (name, owner_id)
        VALUES ($1, $2)
        RETURNING id, name, owner_id, created_at
        "#,
    )
    .bind(payload.name)
    .bind(auth.0.sub)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sqlx::query("INSERT INTO server_members (server_id, user_id) VALUES ($1, $2)")
        .bind(server.id)
        .bind(server.owner_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // create default general channel
    sqlx::query(
        r#"
        INSERT INTO server_members (server_id, user_id)
        VALUES ($1, $2)
        "#,
    )
    .bind(server.id)
    .bind(auth.0.sub)
    .execute(&mut *tx)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tx.commit()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(server))
}

/// Updates a server's name. Only the owner can do this.
async fn _update_server_handle(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<Uuid>,
    auth: AuthUser,
    Json(payload): Json<UpdateServer>,
) -> Result<Json<Server>, (StatusCode, String)> {
    validate(&payload)?;

    let server: Option<Server> = sqlx::query_as(
        r#"
        UPDATE servers SET name = $1
        WHERE id = $2 AND owner_id = $3
        RETURNING id, name, owner_id, created_at
        "#,
    )
    .bind(payload.name)
    .bind(server_id)
    .bind(auth.0.sub)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    server.map(Json).ok_or((
        StatusCode::FORBIDDEN,
        "Server not found or you are not the owner".to_string(),
    ))
}

/// Deletes a server and all associated data. Only the owner can do this.
async fn _delete_server_handle(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<Uuid>,
    auth: AuthUser,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = sqlx::query("DELETE FROM servers WHERE id = $1 AND owner_id = $2")
        .bind(server_id)
        .bind(auth.0.sub)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::FORBIDDEN,
            "Server not found or you are not the owner".to_string(),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Adds the current user to a server's member list.
async fn _join_server_handle(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<Uuid>,
    auth: AuthUser,
) -> Result<Json<ServerMember>, (StatusCode, String)> {
    let existing: Option<ServerMember> = sqlx::query_as(
        "SELECT server_id, user_id, joined_at FROM server_members WHERE server_id = $1 AND user_id = $2",
    )
    .bind(server_id)
    .bind(auth.0.sub)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if existing.is_some() {
        return Err((StatusCode::CONFLICT, "Already a member".to_string()));
    }

    let member: ServerMember = sqlx::query_as(
        "INSERT INTO server_members (server_id, user_id) VALUES ($1, $2) RETURNING server_id, user_id, joined_at",
    )
    .bind(server_id)
    .bind(auth.0.sub)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(member))
}

/// Removes the current user from a server's member list.
async fn _leave_server_handle(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<Uuid>,
    auth: AuthUser,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = sqlx::query("DELETE FROM server_members WHERE server_id = $1 AND user_id = $2")
        .bind(server_id)
        .bind(auth.0.sub)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Member not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Lists all members of a server including their profile data.
async fn _get_members_handle(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<Uuid>,
) -> Result<Json<Vec<User>>, (StatusCode, String)> {
    let members: Vec<User> = sqlx::query_as(
        r#"
        SELECT u.id, u.username, u.email, u.avatar_url, u.status, u.created_at
        FROM users u
        JOIN server_members sm ON u.id = sm.user_id
        WHERE sm.server_id = $1
        ORDER BY sm.joined_at ASC
        "#,
    )
    .bind(server_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(members))
}

/// Creates a new channel in a server. Requires MANAGE_CHANNELS permission.
async fn _create_channel_handle(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<Uuid>,
    auth: AuthUser,
    Json(payload): Json<CreateChannel>,
) -> Result<Json<Channel>, (StatusCode, String)> {
    validate(&payload)?;
    require_permission(
        &state.db,
        server_id,
        auth.0.sub,
        permissions::MANAGE_CHANNELS,
    )
    .await?;

    let channel: Channel = sqlx::query_as(
        r#"
        INSERT INTO channels (server_id, name)
        VALUES ($1, $2)
        RETURNING id, server_id, name, created_at
        "#,
    )
    .bind(server_id)
    .bind(payload.name)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(channel))
}

/// Retrieves all channels in a server ordered by creation time.
async fn _get_channels_handle(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<Uuid>,
) -> Result<Json<Vec<Channel>>, (StatusCode, String)> {
    let channels: Vec<Channel> = sqlx::query_as(
        r#"
        SELECT id, server_id, name, created_at
        FROM channels
        WHERE server_id = $1
        ORDER BY created_at ASC
        "#,
    )
    .bind(server_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(channels))
}

/// Updates a channel's name. Only the server owner can do this.
async fn _update_channel_handle(
    State(state): State<Arc<AppState>>,
    Path((server_id, channel_id)): Path<(Uuid, Uuid)>,
    auth: AuthUser,
    Json(payload): Json<UpdateChannel>,
) -> Result<Json<Channel>, (StatusCode, String)> {
    validate(&payload)?;

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

    let channel: Option<Channel> = sqlx::query_as(
        r#"
        UPDATE channels SET name = $1
        WHERE id = $2 AND server_id = $3
        RETURNING id, server_id, name, created_at
        "#,
    )
    .bind(payload.name)
    .bind(channel_id)
    .bind(server_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    channel
        .map(Json)
        .ok_or((StatusCode::NOT_FOUND, "Channel not found".to_string()))
}

/// Deletes a channel. Only the server owner can do this.
async fn _delete_channel_handle(
    State(state): State<Arc<AppState>>,
    Path((server_id, channel_id)): Path<(Uuid, Uuid)>,
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

    let result = sqlx::query("DELETE FROM channels WHERE id = $1 AND server_id = $2")
        .bind(channel_id)
        .bind(server_id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Channel not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Creates a new role in a server. Requires MANAGE_ROLES permission.
async fn _create_role_handle(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<Uuid>,
    auth: AuthUser,
    Json(payload): Json<CreateRole>,
) -> Result<Json<Role>, (StatusCode, String)> {
    require_permission(&state.db, server_id, auth.0.sub, permissions::MANAGE_ROLES).await?;

    let role: Role = sqlx::query_as(
        r#"
        INSERT INTO roles (server_id, name, permissions)
        VALUES ($1, $2, $3)
        RETURNING id, server_id, name, permissions, created_at
        "#,
    )
    .bind(server_id)
    .bind(payload.name)
    .bind(payload.permissions)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(role))
}

/// Retrieves all roles for a server ordered by creation time.
async fn _get_roles_handle(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<Uuid>,
) -> Result<Json<Vec<Role>>, (StatusCode, String)> {
    let roles: Vec<Role> = sqlx::query_as(
        r#"
        SELECT id, server_id, name, permissions, created_at
        FROM roles
        WHERE server_id = $1
        ORDER BY created_at ASC
        "#,
    )
    .bind(server_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(roles))
}

/// Assigns a role to a user in a server. Requires MANAGE_ROLES permission.
async fn _assign_role_handle(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<Uuid>,
    auth: AuthUser,
    Json(payload): Json<AssignRole>,
) -> Result<Json<UserRole>, (StatusCode, String)> {
    require_permission(&state.db, server_id, auth.0.sub, permissions::MANAGE_ROLES).await?;

    let role_exists: Option<Role> = sqlx::query_as(
        "SELECT id, server_id, name, permissions, created_at FROM roles WHERE id = $1 AND server_id = $2",
    )
    .bind(payload.role_id)
    .bind(server_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if role_exists.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            "Role not found in this server".to_string(),
        ));
    }

    let member_exists: Option<ServerMember> = sqlx::query_as(
        "SELECT server_id, user_id, joined_at FROM server_members WHERE server_id = $1 AND user_id = $2",
    )
    .bind(server_id)
    .bind(payload.user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if member_exists.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            "User is not a member of this server".to_string(),
        ));
    }

    let user_role: UserRole = sqlx::query_as(
        r#"
        INSERT INTO user_roles (server_id, user_id, role_id)
        VALUES ($1, $2, $3)
        ON CONFLICT (server_id, user_id, role_id) DO NOTHING
        RETURNING server_id, user_id, role_id, assigned_at
        "#,
    )
    .bind(server_id)
    .bind(payload.user_id)
    .bind(payload.role_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::CONFLICT, e.to_string()))?;

    Ok(Json(user_role))
}

/// Removes a role from a user in a server.
async fn _revoke_role_handle(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<Uuid>,
    Json(payload): Json<AssignRole>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = sqlx::query(
        "DELETE FROM user_roles WHERE server_id = $1 AND user_id = $2 AND role_id = $3",
    )
    .bind(server_id)
    .bind(payload.user_id)
    .bind(payload.role_id)
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            "User does not have this role".to_string(),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Returns all roles currently held by a specific user in a server.
async fn _get_user_roles_handle(
    State(state): State<Arc<AppState>>,
    Path((server_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<Role>>, (StatusCode, String)> {
    let roles: Vec<Role> = sqlx::query_as(
        r#"
        SELECT r.id, r.server_id, r.name, r.permissions, r.created_at
        FROM roles r
        JOIN user_roles ur ON r.id = ur.role_id
        WHERE ur.server_id = $1 AND ur.user_id = $2
        ORDER BY r.created_at ASC
        "#,
    )
    .bind(server_id)
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(roles))
}

/// Creates an invite link for a server. Only members can create invites.
async fn _create_invite_handle(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<Uuid>,
    auth: AuthUser,
    Json(payload): Json<CreateInvite>,
) -> Result<Json<Invite>, (StatusCode, String)> {
    let is_member: Option<(Uuid,)> =
        sqlx::query_as("SELECT user_id FROM server_members WHERE server_id = $1 AND user_id = $2")
            .bind(server_id)
            .bind(auth.0.sub)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if is_member.is_none() {
        return Err((
            StatusCode::FORBIDDEN,
            "You are not a member of this server".to_string(),
        ));
    }

    let code: String = {
        use rand::Rng;
        rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(8)
            .map(char::from)
            .collect()
    };

    let invite: Invite = sqlx::query_as(
        r#"
        INSERT INTO invites (code, server_id, created_by, max_uses, expires_at)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, code, server_id, created_by, max_uses, uses, expires_at, created_at
        "#,
    )
    .bind(&code)
    .bind(server_id)
    .bind(auth.0.sub)
    .bind(payload.max_uses)
    .bind(payload.expires_at)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(invite))
}

/// Returns invite metadata without consuming a use.
async fn _get_invite_handle(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
) -> Result<Json<Invite>, (StatusCode, String)> {
    let invite: Option<Invite> = sqlx::query_as(
        r#"
        SELECT id, code, server_id, created_by, max_uses, uses, expires_at, created_at
        FROM invites
        WHERE code = $1
        "#,
    )
    .bind(&code)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    invite
        .map(Json)
        .ok_or((StatusCode::NOT_FOUND, "Invite not found".to_string()))
}

/// Uses an invite code to join a server, enforcing expiry and max uses.
async fn _use_invite_handle(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
    auth: AuthUser,
) -> Result<Json<ServerMember>, (StatusCode, String)> {
    let invite: Option<Invite> = sqlx::query_as(
        r#"
        SELECT id, code, server_id, created_by, max_uses, uses, expires_at, created_at
        FROM invites
        WHERE code = $1
        "#,
    )
    .bind(&code)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let invite = invite.ok_or((StatusCode::NOT_FOUND, "Invite not found".to_string()))?;

    if let Some(expires_at) = invite.expires_at {
        if chrono::Utc::now() > expires_at {
            return Err((StatusCode::GONE, "Invite has expired".to_string()));
        }
    }

    if let Some(max_uses) = invite.max_uses {
        if invite.uses >= max_uses {
            return Err((StatusCode::GONE, "Invite has reached its limit".to_string()));
        }
    }

    let existing: Option<ServerMember> = sqlx::query_as(
        "SELECT server_id, user_id, joined_at FROM server_members WHERE server_id = $1 AND user_id = $2",
    )
    .bind(invite.server_id)
    .bind(auth.0.sub)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if existing.is_some() {
        return Err((
            StatusCode::CONFLICT,
            "Already a member of this server".to_string(),
        ));
    }

    sqlx::query("UPDATE invites SET uses = uses + 1 WHERE id = $1")
        .bind(invite.id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let member: ServerMember = sqlx::query_as(
        "INSERT INTO server_members (server_id, user_id) VALUES ($1, $2) RETURNING server_id, user_id, joined_at",
    )
    .bind(invite.server_id)
    .bind(auth.0.sub)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(member))
}
