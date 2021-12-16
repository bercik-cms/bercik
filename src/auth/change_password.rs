use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use axum::extract::{Extension, Json};
use hyper::StatusCode;
use rand_core::OsRng;
use serde::Deserialize;
use sqlx::PgPool;

use super::Claims;

#[derive(Deserialize)]
pub struct ChangePassReq {
    pub new_pass: String,
    pub old_pass: String, // TODO: check old password
}

pub async fn change_password(
    claims: Claims,
    Json(req): Json<ChangePassReq>,
    Extension(db_pool): Extension<PgPool>,
) -> Result<String, (StatusCode, String)> {
    let username = &claims.username;
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(req.new_pass.as_bytes(), &salt)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Could not hash password".into(),
            )
        })?
        .to_string();

    sqlx::query("UPDATE TABLE __B_users SET password_hash = $1 WHERE username = $2")
        .bind(password_hash)
        .bind(username)
        .execute(&db_pool)
        .await
        .map_err(|it| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("ERROR updating password: {}", it),
            )
        })?;

    Ok("ok".into())
}
