use std::collections::HashMap;

use crate::models::{Message, ServerSignal};
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast, mpsc};
use uuid::Uuid;

pub struct VoiceSession {
    pub user_id: Uuid,
    pub username: String,
    /// Server sends signaling messages down to this channel to the WS task.
    pub tx: mpsc::Sender<ServerSignal>,
}

/// All Active sessions for one voice channel.
pub type VoiceRoom = Vec<Arc<VoiceSession>>;

pub struct AppState {
    pub db: sqlx::PgPool,
    pub tx: broadcast::Sender<Message>,
    pub presence: RwLock<HashMap<Uuid, String>>,
    pub voice_rooms: RwLock<HashMap<Uuid, VoiceRoom>>,
}
