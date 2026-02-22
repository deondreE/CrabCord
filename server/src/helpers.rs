use axum::http::StatusCode;
use uuid::Uuid;

use crate::models::permissions;

pub async fn require_permission(
    db: &sqlx::PgPool,
    server_id: Uuid,
    user_id: Uuid,
    required: i64,
) -> Result<(), (StatusCode, String)> {
    // Owner bypasses all permission immediatly "ADMIN"
    let is_owner: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM servers WHERE id = $1 AND owner_id = $2")
            .bind(server_id)
            .bind(user_id)
            .fetch_optional(db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if is_owner.is_some() {
        return Ok(());
    }

    let row: Option<(i64,)> = sqlx::query_as(
        r#"
        SELECT COALESCE(BIT_OR(r.permissions), 0)
        FROM roles r
        JOIN user_roles ur ON r.id = ur.role_id
        WHERE ur.server_id = $1 AND ur.user_id = $2
        "#,
    )
    .bind(server_id)
    .bind(user_id)
    .fetch_optional(db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let combined = row.map(|(p,)| p).unwrap_or(0);

    if combined & permissions::ADMINISTRATOR != 0 || combined & required != 0 {
        Ok(())
    } else {
        Err((
            StatusCode::FORBIDDEN,
            "Insufficient permissions".to_string(),
        ))
    }
}
