use crate::err_utils::to_internal;
use axum::extract::{Extension, Json};
use hyper::StatusCode;
use serde::Deserialize;
use sqlx::PgPool;

use super::Claims;

#[derive(Deserialize)]
pub struct DeleteUserRequest {
    pub username: String,
}

pub async fn delete_user_route(
    claims: Claims,
    Extension(db_pool): Extension<PgPool>,
    Json(req): Json<DeleteUserRequest>,
) -> Result<String, (StatusCode, String)> {
    claims.must_be_admin()?;

    super::delete_user_service::delete_user_service(&db_pool, &req.username)
        .await
        .map_err(to_internal)?;

    Ok("Ok".into())
}
