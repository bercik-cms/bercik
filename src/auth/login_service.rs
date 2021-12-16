use crate::auth::Claims;

use super::login_route::{LoginRequest, LoginResponse};
use anyhow::{anyhow, Result};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use once_cell::sync::Lazy;
use sqlx::{Executor, FromRow, PgPool, Postgres};

#[derive(Debug)]
pub struct Keys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey<'static>,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret).into_static(),
        }
    }
}

pub static KEYS: Lazy<Keys> = Lazy::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    Keys::new(secret.as_bytes())
});

#[derive(FromRow)]
struct UserInfo {
    pub username: String,
    pub password_hash: String,
    pub user_group: String,
}

pub async fn login(req: &LoginRequest, db_pool: &PgPool) -> Result<LoginResponse> {
    let user_info: UserInfo = sqlx::query_as::<Postgres, UserInfo>(
        "SELECT username, password_hash, user_group from __B_users where username=$1",
    )
    .bind(&req.username)
    .fetch_one(db_pool)
    .await?;

    let parsed_hash =
        PasswordHash::new(&user_info.password_hash).map_err(|it| anyhow!("{}", it))?;

    Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .map_err(|it| anyhow!("{}", it))?;

    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::days(7))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        username: user_info.username.to_owned(),
        user_group: user_info.user_group.to_owned(),
        exp: expiration as usize,
    };

    let token = encode(&Header::default(), &claims, &KEYS.encoding)?;

    Ok(LoginResponse { token })
}
