mod models;
mod state;

use axum::{
    Json, Router,
    extract::{Path, State, ws::close_code::AWAY},
    http::StatusCode,
    routing::{get, post},
};
use dotenvy::dotenv;
use serde::ser;
use sqlx::postgres::PgPoolOptions;
use std::{env, mem, sync::Arc};
use tokio::sync::broadcast;

use crate::{
    models::{
        AssignRole, Channel, CreateChannel, CreateMessage, CreateRole, CreateServer, CreateUser,
        JoinServer, Message, Role, Server, ServerMember, User, UserRole,
    },
    state::AppState,
};

use uuid::Uuid;

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

    let (tx, _rx) = broadcast::channel::<Message>(100);

    let shared_state = Arc::new(AppState { db: pool, tx });

    let app = Router::new()
        .route("/users", post(_create_user_handle))
        .route("/servers", post(_create_server_handle))
        .route("/servers/:server_id/join", post(_join_server_handle))
        .route("/servers/:server_id/leave", post(_leave_server_handle))
        .route("/servers/:server_id/members", get(_get_members_handle))
        .route(
            "/servers/:server_id/members/:user_id/roles",
            get(_get_user_roles_handle),
        )
        .route("/servers/:server_id/channels", post(_create_channel_handle))
        .route("/servers/:server_id/channels", get(_get_channels_handle))
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
        .route(
            "/channels/:channel_id/messages",
            post(_send_messages_handle),
        )
        .route("/channels/:channel_id/messages", get(_get_messages_handle))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("🚀 Server active at http://localhost:3000");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn _send_messages_handle(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateMessage>,
) -> Result<Json<Message>, (StatusCode, String)> {
    let message: Message = sqlx::query_as(
        r#"
        INSERT INTO messages (channel_id, user_id, content)
        VALUES ($1, $2, $3)
        RETURNING id, channel_id, user_id,
            (SELECT username FROM users WHERE id = user_id) AS username,
            content, created_at
        "#,
    )
    .bind(payload.channel_id)
    .bind(payload.user_id)
    .bind(payload.content)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let _ = state.tx.send(message.clone());
    Ok(Json(message))
}

async fn _create_user_handle(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<User>, (StatusCode, String)> {
    let user: User = sqlx::query_as(
        r#"
        INSERT INTO users (username, email)
        VALUES ($1, $2)
        RETURNING id, username, email, created_at
        "#,
    )
    .bind(payload.username)
    .bind(payload.email)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(user))
}

async fn _create_server_handle(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateServer>,
) -> Result<Json<Server>, (StatusCode, String)> {
    let server: Server = sqlx::query_as(
        r#"
            INSERT INTO servers (name, owner_id)
            VALUES ($1, $2)
            RETURNING id, name, owner_id, created_at
        "#,
    )
    .bind(payload.name)
    .bind(payload.owner_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sqlx::query(
        r#"
        INSERT INTO server_members (server_id, user_id)
        VALUES ($1, $2)
        "#,
    )
    .bind(server.id)
    .bind(server.owner_id)
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(server))
}

async fn _create_role_handle(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<Uuid>,
    Json(payload): Json<CreateRole>,
) -> Result<Json<Role>, (StatusCode, String)> {
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

async fn _assign_role_handle(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<Uuid>,
    Json(payload): Json<AssignRole>,
) -> Result<Json<UserRole>, (StatusCode, String)> {
    let role_exists: Option<Role> = sqlx::query_as(
        "SELECT id, server_id, name, permissions, created_at FROM roles where id = $1 AND server_id = $2"
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
        "SELECT id, server_id, user_id, joined_at FROM server_members WHERE server_id = $1 AND user_id = $2"
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

async fn _revoke_role_handle(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<Uuid>,
    Json(payload): Json<AssignRole>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = sqlx::query(
        r#"
           DELETE FROM user_roles
           WHERE server_id = $1 AND user_id $2 AND role_id = $3
        "#,
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

async fn _create_channel_handle(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<Uuid>,
    Json(payload): Json<CreateChannel>,
) -> Result<Json<Channel>, (StatusCode, String)> {
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

async fn _get_channels_handle(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<Uuid>,
) -> Result<Json<Vec<Channel>>, (StatusCode, String)> {
    let channels: Vec<Channel> = sqlx::query_as(
        r#"
            SELECT id, server_id, name, created_at
            FROM channels
            WHERE server_id = $1
            ORDER by created_at ASC
        "#,
    )
    .bind(server_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(channels))
}

async fn _join_server_handle(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<Uuid>,
    Json(payload): Json<JoinServer>,
) -> Result<Json<ServerMember>, (StatusCode, String)> {
    let existing: Option<ServerMember> = sqlx::query_as(
        "SELECT server_id, user_id, joined_at FROM server_members WHERE server_id = $1 AND user_id = $2"
    )
    .bind(server_id)
    .bind(payload.user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(member) = existing {
        return Err((StatusCode::CONFLICT, "Already a member".to_string()));
    }

    let member: ServerMember = sqlx::query_as(
        r#"
            INSERT INTO server_members (server_id, user_id)
            VALUES ($1, $2)
            ON CONFLICT (server_id, user_id) DO NOTHING
            RETURNING server_id, user_id, joined_at
            "#,
    )
    .bind(server_id)
    .bind(payload.user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::CONFLICT, e.to_string()))?;

    Ok(Json(member))
}
async fn _leave_server_handle(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<Uuid>,
    Json(payload): Json<JoinServer>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = sqlx::query(
        r#"
          DELETE FROM server_members
          WHERE server_id = $1 AND user_id = $2
        "#,
    )
    .bind(server_id)
    .bind(payload.user_id)
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Member not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

async fn _get_members_handle(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<Uuid>,
) -> Result<Json<Vec<User>>, (StatusCode, String)> {
    let members: Vec<User> = sqlx::query_as(
        r#"
        SELECT u.id, u.username, u.email, u.created_at
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

async fn _get_messages_handle(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<Uuid>,
) -> Result<Json<Vec<Message>>, (StatusCode, String)> {
    let messages: Vec<Message> = sqlx::query_as(
        r#"  SELECT m.id, m.server_id, m.user_id, u.username, m.content, m.created_at
            FROM messages m
            JOIN users u ON m.user_id = u.id
            WHERE m.channel_id = $1
            ORDER BY m.created_at DESC LIMIT 50
        "#,
    )
    .bind(server_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(messages))
}
