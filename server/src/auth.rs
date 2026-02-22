use std::time::{SystemTime, UNIX_EPOCH};

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const JWT_EXP_SECONDS: u64 = 60 * 60 * 24;
const REFRESH_EXPIRY_DAYS: i64 = 30;

#[derive(Deserialize, Serialize)]
pub struct Claims {
    pub sub: Uuid,
    pub username: String,
    pub exp: u64,
}

fn jwt_secret() -> String {
    std::env::var("JWT_SECRET").expect("JWT_SECRET must be set")
}

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    Ok(argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string())
}

pub fn create_refresh_token() -> String {
    use rand::Rng;
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(64)
        .map(char::from)
        .collect()
}

pub fn refresh_expiry() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now() + chrono::Duration::days(REFRESH_EXPIRY_DAYS)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hash)?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub fn create_token(user_id: Uuid, username: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let exp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + JWT_EXP_SECONDS;

    let claims = Claims {
        sub: user_id,
        username: username.to_string(),
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret().as_bytes()),
    )
}

pub fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret().as_bytes()),
        &Validation::default(),
    )?;

    Ok(data.claims)
}
