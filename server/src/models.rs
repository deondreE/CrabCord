use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub avatar_url: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct DirectMessage {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub receiver_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
    pub read_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Validate)]
pub struct CreateDirectMessage {
    #[validate(length(
        min = 1,
        max = 2000,
        message = "Message must be between 1 and 2000 characters"
    ))]
    pub content: String,
}

#[derive(Deserialize, Validate)]
pub struct UpdateDirectMessage {
    #[validate(length(
        min = 1,
        max = 2000,
        message = "Message must be between 1 and 2000 characters"
    ))]
    pub content: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum UserStatus {
    Online,
    Idle,
    DoNotDisturb,
    Offline,
}

impl std::fmt::Display for UserStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            UserStatus::Online => write!(f, "online"),
            UserStatus::Idle => write!(f, "idle"),
            UserStatus::DoNotDisturb => write!(f, "dnd"),
            UserStatus::Offline => write!(f, "offline"),
        }
    }
}

#[derive(Deserialize)]
pub struct UpdateStatus {
    pub status: String,
}

#[derive(Deserialize, Validate)]
pub struct UserSearchQuery {
    pub username: String,
}

#[derive(Deserialize, Validate)]
pub struct CreateUser {
    #[validate(length(
        min = 3,
        max = 32,
        message = "Username must be between 3 and 32 characters"
    ))]
    pub username: String,
    #[validate(email(message = "Invalid email address"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
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
    pub refresh_token: String,
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

#[derive(Deserialize, Validate)]
pub struct CreateChannel {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Channel name must be between 1 and 100 characters"
    ))]
    pub name: String,
}

#[derive(Deserialize, Validate)]
pub struct UpdateChannel {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Channel name must be between 1 and 100 characters"
    ))]
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct ServerMember {
    pub server_id: Uuid,
    pub user_id: Uuid,
    pub joined_at: DateTime<Utc>,
}

#[derive(Deserialize, Validate)]
pub struct CreateServer {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Server name must be between 1 and 100 characters"
    ))]
    pub name: String,
}

#[derive(Deserialize, Validate)]
pub struct UpdateServer {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Server name must be between 1 and 100 characters"
    ))]
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ReactionSummary {
    pub emoji_id: String,
    pub count: i64,
    pub user_ids: Vec<Uuid>,
}

#[derive(sqlx::FromRow)]
pub struct MessageRow {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub user_id: Uuid,
    pub username: Option<String>,
    pub content: String,
    #[sqlx(rename = "reactions!")]
    pub reactions: sqlx::types::Json<Vec<ReactionSummary>>,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub user_id: Uuid,
    pub username: Option<String>,
    pub content: String,
    pub reactions: Vec<ReactionSummary>,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
}

impl From<MessageRow> for Message {
    fn from(row: MessageRow) -> Self {
        Self {
            id: row.id,
            channel_id: row.channel_id,
            user_id: row.user_id,
            username: row.username,
            content: row.content,
            reactions: row.reactions.0,
            created_at: row.created_at,
            edited_at: row.edited_at,
        }
    }
}

impl From<Message> for tokio::sync::broadcast::error::SendError<Message> {
    fn from(_: Message) -> Self {
        unimplemented!();
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateMessage {
    #[validate(length(
        min = 1,
        max = 2000,
        message = "Message must be between 1 and 2000 characters"
    ))]
    pub content: String,
}

#[derive(Deserialize, Validate)]
pub struct CreateMessage {
    #[validate(length(
        min = 1,
        max = 2000,
        message = "Message must be between 1 and 2000 characters"
    ))]
    pub content: String,
}

pub mod permissions {
    pub const VIEW_CHANNELS: i64 = 1 << 0;
    pub const SEND_MESSAGES: i64 = 1 << 1;
    pub const MANAGE_MESSAGES: i64 = 1 << 2;
    pub const MANAGE_CHANNELS: i64 = 1 << 3;
    pub const MANAGE_ROLES: i64 = 1 << 4;
    pub const KICK_MEMBERS: i64 = 1 << 5;
    pub const BAN_MEMBERS: i64 = 1 << 6;
    pub const ADMINISTRATOR: i64 = 1 << 7;
}

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

#[derive(Deserialize, Validate)]
pub struct UpdateProfile {
    #[validate(length(
        min = 3,
        max = 32,
        message = "Username must be between 3 and 32 characters"
    ))]
    pub username: Option<String>,
    #[validate(email(message = "Invalid email address"))]
    pub email: Option<String>,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

/// Avoids duplicating the MessageRow -> Message conversion at every call site.
pub async fn fetch_message(
    db: &sqlx::PgPool,
    query: &str,
    message_id: uuid::Uuid,
) -> Result<Message, sqlx::Error> {
    let row: MessageRow = sqlx::query_as(query).bind(message_id).fetch_one(db).await?;
    Ok(row.into())
}

/// Helper to fetch many messages (e.g. for get_messages).
pub async fn fetch_messages(
    db: &sqlx::PgPool,
    query: &str,
    channel_id: uuid::Uuid,
) -> Result<Vec<Message>, sqlx::Error> {
    let rows: Vec<MessageRow> = sqlx::query_as(query).bind(channel_id).fetch_all(db).await?;
    Ok(rows.into_iter().map(Message::from).collect())
}
