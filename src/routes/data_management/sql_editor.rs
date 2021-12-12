use crate::err_utils::to_internal;
use crate::types::arbitrary_sql_array_row::ArbitrarySqlArrayRowsAndNames;
use axum::extract::Extension;
use axum::extract::Json;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Deserialize)]
pub struct ExecuteQueriesRequest {
    pub queries: Vec<String>,
    pub diff_query: String,
    pub should_diff_query: bool,
    pub should_diff_mermaid: bool,
    pub execute: bool,
}

#[derive(Debug, Serialize)]
pub struct ExecuteQueriesResponse {
    pub query_results: Vec<ArbitrarySqlArrayRowsAndNames>,
    pub query_diff: Option<Vec<ArbitrarySqlArrayRowsAndNames>>,
    pub mermaid_diff: Option<Vec<String>>,
}

pub async fn execute_queries(
    Json(req): Json<ExecuteQueriesRequest>,
    Extension(db_pool): Extension<PgPool>,
) -> Result<Json<ExecuteQueriesResponse>, (StatusCode, String)> {
    use crate::services::sql_execution::execute_queries;
    let result = execute_queries(&db_pool, &req).await.map_err(to_internal)?;
    Ok(Json(result))
}
