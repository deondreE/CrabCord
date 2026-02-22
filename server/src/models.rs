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
pub struct Server {
    pub id: Uuid,
    pub name: String,
    pub owner_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Channel {
    pub id: Uuid,
    pub server_id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct CreateChannel {
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct ServerMember {
    pub server_id: Uuid,
    pub user_id: Uuid,
    pub joined_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct CreateServer {
    pub name: String,
    pub owner_id: Uuid,
}

#[derive(Deserialize)]
pub struct JoinServer {
    pub user_id: Uuid,
}

#[derive(Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Message {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub user_id: Uuid,
    pub username: Option<String>,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMessage {
    pub channel_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
}
