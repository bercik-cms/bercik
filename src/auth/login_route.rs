use super::login_service::login as login_service;
use super::Claims;
use crate::err_utils::to_internal;
use axum::extract::Extension;
use axum::extract::Json;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub claims: Claims,
}

pub async fn login(
    Json(login_info): Json<LoginRequest>,
    Extension(db_pool): Extension<PgPool>,
) -> Result<Json<LoginResponse>, (StatusCode, String)> {
    Ok(Json(
        login_service(&login_info, &db_pool)
            .await
            .map_err(to_internal)?,
    ))
}
