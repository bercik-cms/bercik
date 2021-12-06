use axum::extract::Extension;
use axum::http::StatusCode;
use axum::Json;
use sqlx::PgPool;

use crate::types::table_info::TableInfo;

pub async fn get_table_info(
    Extension(db_pool): Extension<PgPool>,
) -> Result<Json<Vec<TableInfo>>, (StatusCode, String)> {
    use crate::err_utils::to_internal;
    use crate::services::schema_info::table_info;

    let info = table_info::get_table_info(&db_pool)
        .await
        .map_err(to_internal)?;

    Ok(Json(info))
}
