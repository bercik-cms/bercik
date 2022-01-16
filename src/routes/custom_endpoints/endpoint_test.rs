use std::collections::HashMap;

use super::endpoint_crud::CreateEndpointRequest;
use crate::err_utils::to_internal;
use crate::services::endpoints::endpoint_test::test_endpoint as test_endpoint_service;
use crate::{algorithms::sql_variable_parser::EndpointInfo, auth::Claims};
use axum::{
    body::HttpBody,
    extract::{Extension, Json},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Serialize)]
pub struct EndpointTestResult {
    ok: bool,
    msg: String,
}

#[derive(Deserialize)]
pub struct EndpointTestRequest {
    pub create_req: CreateEndpointRequest,
    pub req_variables: HashMap<String, String>,
}

pub async fn endpoint_test(
    claims: Claims,
    Json(req): Json<super::endpoint_test::EndpointTestRequest>,
    Extension(db_pool): Extension<PgPool>,
) -> Result<Json<EndpointTestResult>, (StatusCode, String)> {
    claims.must_be_admin()?;

    let parsed_endpoints_result = req
        .create_req
        .endpoints_info
        .into_iter()
        .map(EndpointInfo::from_request)
        .collect::<anyhow::Result<Vec<EndpointInfo>>>();

    if let Err(err) = parsed_endpoints_result {
        return Ok(Json(EndpointTestResult {
            ok: false,
            msg: format!("{}", err),
        }));
    }

    let result = test_endpoint_service(
        &db_pool,
        parsed_endpoints_result.unwrap(),
        req.req_variables,
    )
    .await;

    match result {
        Ok(result) => Ok(Json(EndpointTestResult {
            ok: true,
            msg: serde_json::to_string_pretty(&result).unwrap(),
        })),
        Err(err) => Ok(Json(EndpointTestResult {
            ok: false,
            msg: format!("{}", err),
        })),
    }
}
