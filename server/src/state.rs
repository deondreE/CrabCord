use std::collections::HashMap;

use crate::models::Message;
use tokio::sync::{RwLock, broadcast};
use uuid::Uuid;

pub struct AppState {
    pub db: sqlx::PgPool,
    pub tx: broadcast::Sender<Message>,
    pub presence: RwLock<HashMap<Uuid, String>>,
}
