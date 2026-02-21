mod models;
mod state;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::{env, sync::Arc};
use tokio::sync::broadcast;

use crate::{
    models::{CreateMessage, CreateUser, Message, User},
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
        CREATE TABLE IF NOT EXISTS messages (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            server_id UUID NOT NULL,
            user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            content TEXT NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT now()
        );
        "#,
    )
    .execute(&pool)
    .await?;

    let (tx, _rx) = broadcast::channel::<Message>(100);

    let shared_state = Arc::new(AppState { db: pool, tx });

    let app = Router::new()
        .route("/users", post(handle_create_user))
        .route("/messages", post(handle_send_message))
        .route("/messages/:server_id", get(handle_get_messages))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("🚀 Server active at http://localhost:3000");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn handle_send_message(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateMessage>,
) -> Result<Json<Message>, (StatusCode, String)> {
    let message: Message = sqlx::query_as(
        r#"
        INSERT INTO messages (server_id, user_id, content)
        VALUES ($1, $2, $3)
        RETURNING id, server_id, user_id,
            (SELECT username FROM users WHERE id = user_id) AS username,
            content, created_at
        "#,
    )
    .bind(payload.server_id)
    .bind(payload.user_id)
    .bind(payload.content)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let _ = state.tx.send(message.clone());
    Ok(Json(message))
}

async fn handle_create_user(
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

async fn handle_get_messages(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<Uuid>,
) -> Result<Json<Vec<Message>>, (StatusCode, String)> {
    let messages: Vec<Message> = sqlx::query_as(
        "  SELECT m.id, m.server_id, m.user_id, u.username, m.content, m.created_at
              FROM messages m
            JOIN users u ON m.user_id = u.id
              WHERE m.server_id = $1
              ORDER BY m.created_at DESC LIMIT 50",
    )
    .bind(server_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(messages))
}
