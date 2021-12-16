use crate::auth::Claims;
use axum::{
    extract::{Extension, Json},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use super::create_users_service::UsernamePass;

#[derive(Deserialize)]
pub struct CreateUsersRequest {
    pub username: String,
    pub amount: usize,
    pub user_group: String,
}

#[derive(Serialize)]
pub struct CreateUsersResponse {
    pub new_users: Vec<UsernamePass>,
}

pub async fn create_users(
    Json(req): Json<CreateUsersRequest>,
    Extension(db_pool): Extension<PgPool>,
    claims: Claims,
) -> Result<Json<CreateUsersResponse>, (StatusCode, String)> {
    use super::create_users_service::create_users as c_users_service;
    use crate::err_utils::to_internal;

    claims.must_be_admin()?;
    let user_passwords = c_users_service(&req, &db_pool).await.map_err(to_internal)?;

    Ok(Json(CreateUsersResponse {
        new_users: user_passwords,
    }))
}
