use crate::models::Message;
use tokio::sync::broadcast;

pub struct AppState {
    pub db: sqlx::PgPool,
    pub tx: broadcast::Sender<Message>,
}
