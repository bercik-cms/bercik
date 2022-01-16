use super::get_users_service::{get_users_service, UserInfo};
use super::Claims;
use crate::err_utils::to_internal;
use axum::extract::Extension;
use axum::http::StatusCode;
use axum::Json;
use sqlx::PgPool;

pub async fn get_users_route(
    claims: Claims,
    Extension(db_pool): Extension<PgPool>,
) -> Result<Json<Vec<UserInfo>>, (StatusCode, String)> {
    claims.must_be_admin()?;
    Ok(Json(
        get_users_service(&db_pool).await.map_err(to_internal)?,
    ))
}
