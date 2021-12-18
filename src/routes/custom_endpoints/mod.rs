pub mod endpoint_crud;
pub mod endpoint_test;

use crate::algorithms::endpoint_execution::ExecutionResult;
use crate::services::endpoints::endpoint_execution::execute_endpoint;
use crate::{algorithms::sql_variable_parser::EndpointInfo, err_utils::to_internal};
use axum::{
    body::HttpBody,
    extract::{
        rejection::{FormRejection, JsonRejection},
        Extension, Form, Json, Path,
    },
    http::StatusCode,
};
use sqlx::PgPool;
use sqlx::Postgres;
use sqlx::{Executor, FromRow};
use std::collections::HashMap;

#[derive(FromRow)]
pub struct EndpointExecutionInfo {
    pub req_method: String,
    pub handler_info: String,
    pub allowed_groups: String,
}

pub async fn custom_endpoint(
    path: Path<String>,
    Extension(db_pool): Extension<PgPool>,
    form_result: Result<Form<HashMap<String, String>>, FormRejection>,
    json_result: Result<Json<HashMap<String, String>>, JsonRejection>,
) -> Result<Json<HashMap<String, ExecutionResult>>, (StatusCode, String)> {
    let arguments = match (form_result, json_result) {
        (Err(form_err), Err(json_err)) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Couldn't parse form ({}) or json ({})", form_err, json_err),
            ))
        }
        (Ok(Form(form)), Err(_)) => form,
        (Err(_), Ok(Json(json))) => json,

        // If both json and form, ignore json.
        // Happens when GET request has query params
        // and json body.
        (Ok(Form(form)), Ok(_json)) => form,
    };

    let path = path.to_string();

    dbg!(&path, &arguments);

    let endpoint_info = sqlx::query_as::<Postgres, EndpointExecutionInfo>(
        "SELECT req_method, handler_info, allowed_groups FROM __B_endpoints WHERE req_path=$1",
    )
    .bind(&path)
    .fetch_one(&db_pool)
    .await
    .map_err(to_internal)?;

    let result = execute_endpoint(&db_pool, endpoint_info, arguments)
        .await
        .map_err(to_internal)?;

    Ok(Json(result))
}
