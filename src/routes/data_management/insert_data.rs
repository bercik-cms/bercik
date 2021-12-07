use crate::services::data_management::insert_data::insert_data as insert_data_service;
use axum::extract::{Extension, Json};
use hyper::StatusCode;
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Deserialize)]
pub struct ColumnValue {
    pub value: String,
    pub use_default: bool,
    pub use_null: bool,
}

#[derive(Deserialize)]
pub struct InsertDataRequest {
    pub table_name: String,
    pub values: Vec<ColumnValue>,
}

pub async fn insert_data(
    Extension(db_pool): Extension<PgPool>,
    Json(data): Json<InsertDataRequest>,
) -> Result<(), (StatusCode, String)> {
    insert_data_service(&db_pool, data)
        .await
        .map_err(crate::err_utils::to_internal)?;
    Ok(())
}
