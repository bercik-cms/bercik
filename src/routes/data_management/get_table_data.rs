use crate::err_utils::to_internal;
use crate::services::data_management::get_table_data::get_table_data as table_data_service;
use crate::types::arbitrary_sql_array_row::ArbitrarySqlArrayRowsAndNames;
use axum::extract::Extension;
use axum::extract::Json;
use axum::http::StatusCode;
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Deserialize, Debug)]
#[serde(tag = "type", content = "content")]
pub enum WhereClause {
    None,
    ColumnEquals { col_name: String, equals: String },
    Custom(String),
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", content = "content")]
pub enum Sorting {
    None,
    ColumnDescending(String),
    ColumnAscending(String),
    CustomExpression(String),
}

#[derive(Deserialize, Debug)]
pub struct GetTableDataRequest {
    pub table_name: String,
    pub where_clause: WhereClause,
    pub sorting: Sorting,
    pub page: Option<u32>,
}

pub async fn get_table_data(
    Extension(db_pool): Extension<PgPool>,
    Json(req): Json<GetTableDataRequest>,
) -> Result<Json<ArbitrarySqlArrayRowsAndNames>, (StatusCode, String)> {
    let data = table_data_service(&db_pool, &req)
        .await
        .map_err(to_internal)?;

    Ok(Json(data))
}
