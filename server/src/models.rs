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

#[derive(Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
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

#[derive(Deserialize)]
pub struct UpdateChannel {
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
}

#[derive(Deserialize)]
pub struct UpdateServer {
    pub name: String,
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
    pub edited_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMessage {
    pub content: String,
}

#[derive(Deserialize)]
pub struct CreateMessage {
    pub content: String,
}

pub mod permissions {
    pub const VIEW_CHANNELS: i64 = 1 << 0; // 1
    pub const SEND_MESSAGES: i64 = 1 << 1; // 2
    pub const MANAGE_MESSAGES: i64 = 1 << 2; // 4
    pub const MANAGE_CHANNELS: i64 = 1 << 3; // 8
    pub const MANAGE_ROLES: i64 = 1 << 4; // 16
    pub const KICK_MEMBERS: i64 = 1 << 5; // 32
    pub const BAN_MEMBERS: i64 = 1 << 6; // 64
    pub const ADMINISTRATOR: i64 = 1 << 7; // 128 ==> Bypass all permissions
}

// x << n :=> x * n^2

#[derive(Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Role {
    pub id: Uuid,
    pub server_id: Uuid,
    pub name: String,
    pub permissions: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct CreateRole {
    pub name: String,
    pub permissions: i64,
}

#[derive(Deserialize)]
pub struct AssignRole {
    pub user_id: Uuid,
    pub role_id: Uuid,
}

#[derive(Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct UserRole {
    pub server_id: Uuid,
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub assigned_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Invite {
    pub id: Uuid,
    pub code: String,
    pub server_id: Uuid,
    pub created_by: Uuid,
    pub max_uses: Option<i32>,
    pub uses: i32,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct CreateInvite {
    pub max_uses: Option<i32>,
    pub expires_at: Option<DateTime<Utc>>,
}
