pub mod endpoint_crud;
pub mod endpoint_test;

use crate::algorithms::endpoint_execution::ExecutionResult;
use crate::auth::Claims;
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
    claims_opt: Option<Claims>,
) -> Result<Json<HashMap<String, Vec<ExecutionResult>>>, (StatusCode, String)> {
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

    dbg!(&path, &arguments, &claims_opt);

    let endpoint_info = sqlx::query_as::<Postgres, EndpointExecutionInfo>(
        "SELECT req_method, handler_info, allowed_groups FROM __B_endpoints WHERE req_path=$1",
    )
    .bind(&path)
    .fetch_one(&db_pool)
    .await
    .map_err(to_internal)?;

    let allowed_groups =
        serde_json::from_str::<Vec<String>>(&endpoint_info.allowed_groups).map_err(to_internal)?;

    dbg!(&allowed_groups);

    println!(
        "can call endpoint: {}",
        can_call_endpoint(claims_opt.as_ref(), allowed_groups.clone())
    );

    if !can_call_endpoint(claims_opt.as_ref(), allowed_groups) {
        return Err((
            StatusCode::UNAUTHORIZED,
            "You are not authorized to call this endpoint".into(),
        ));
    }

    let result = execute_endpoint(&db_pool, endpoint_info, arguments)
        .await
        .map_err(to_internal)?;

    Ok(Json(result))
}

fn can_call_endpoint(claims_opt: Option<&Claims>, allowed_groups: Vec<String>) -> bool {
    for group in &allowed_groups {
        if group == "PUBLIC" {
            return true;
        }
    }

    if claims_opt.is_none() {
        return false;
    }

    let current_group = claims_opt.unwrap().user_group();

    if current_group == "ADMIN" {
        return true;
    }

    for group in &allowed_groups {
        if current_group == group {
            return true;
        }
    }

    false
}
