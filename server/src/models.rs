use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub email: String,
}

#[derive(Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Message {
    pub id: Uuid,
    pub server_id: Uuid,
    pub user_id: Uuid,
    pub username: Option<String>,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct SendMessage {
    pub server_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateMessage {
    pub server_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
}
