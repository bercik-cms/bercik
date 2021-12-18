use crate::{auth::Claims, err_utils::to_internal};
use axum::extract::Extension;
use axum::Json;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::algorithms::sql_variable_parser::{EndpointInfo, EndpointInfoCreateRequest};

use crate::services::endpoints::crud_endoints as endpoint_services;

#[derive(Deserialize, Serialize)]
pub enum CreateEndpointMethod {
    GET,
    POST,
    ANY,
}

impl CreateEndpointMethod {
    pub fn to_string(&self) -> &'static str {
        match self {
            &Self::GET => "GET",
            &Self::POST => "POST",
            &Self::ANY => "ANY",
        }
    }

    pub fn from_str(s: &str) -> anyhow::Result<Self> {
        match s {
            "GET" => Ok(Self::GET),
            "POST" => Ok(Self::POST),
            "ANY" => Ok(Self::ANY),
            a => Err(anyhow::anyhow!("{} is not a method", a)),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct CreateEndpointRequest {
    pub path: String,
    pub method: CreateEndpointMethod,
    pub endpoints_info: Vec<EndpointInfoCreateRequest>,
    pub allowed_groups: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub struct GetEndpointInfo {
    pub id: i32,
    pub path: String,
    pub method: CreateEndpointMethod,
    pub endpoints_info: Vec<EndpointInfoCreateRequest>,
    pub allowed_groups: Vec<String>,
}

pub async fn create_endpoint(
    Extension(db_pool): Extension<PgPool>,
    Json(req): Json<CreateEndpointRequest>,
    claims: Claims,
) -> Result<(), (StatusCode, String)> {
    claims.must_be_admin()?;

    endpoint_services::create_endpoint(&db_pool, req)
        .await
        .map_err(to_internal)?;

    Ok(())
}

pub async fn get_endpoints(
    Extension(db_pool): Extension<PgPool>,
    claims: Claims,
) -> Result<Json<Vec<GetEndpointInfo>>, (StatusCode, String)> {
    claims.must_be_admin()?;
    Ok(Json(
        endpoint_services::get_endpoints(&db_pool)
            .await
            .map_err(to_internal)?,
    ))
}

#[derive(Deserialize, Serialize)]
pub struct UpdateEndpointRequest {
    pub id: i32,
    pub path: String,
    pub method: CreateEndpointMethod,
    pub endpoints_info: Vec<EndpointInfoCreateRequest>,
    pub allowed_groups: Vec<String>,
}

impl UpdateEndpointRequest {
    pub fn to_create_and_id(self) -> (CreateEndpointRequest, i32) {
        (
            CreateEndpointRequest {
                path: self.path,
                method: self.method,
                endpoints_info: self.endpoints_info,
                allowed_groups: self.allowed_groups,
            },
            self.id,
        )
    }
}

pub async fn update_endpoint(
    Extension(db_pool): Extension<PgPool>,
    Json(update_req): Json<UpdateEndpointRequest>,
    claims: Claims,
) -> Result<(), (StatusCode, String)> {
    claims.must_be_admin()?;
    let (req, endpoint_id) = update_req.to_create_and_id();
    endpoint_services::update_endpoint(&db_pool, endpoint_id, req)
        .await
        .map_err(to_internal)?;
    Ok(())
}

#[derive(Deserialize)]
pub struct DeleteEndpointRequest {
    id: i32,
}

pub async fn delete_endpoint(
    Extension(db_pool): Extension<PgPool>,
    Json(req): Json<DeleteEndpointRequest>,
    claims: Claims,
) -> Result<(), (StatusCode, String)> {
    claims.must_be_admin()?;
    endpoint_services::delete_endpoint(&db_pool, req.id)
        .await
        .map_err(to_internal)?;

    Ok(())
}
