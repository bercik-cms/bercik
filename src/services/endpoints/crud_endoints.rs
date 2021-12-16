use crate::routes::custom_endpoints::endpoint_crud::CreateEndpointRequest;
use crate::{
    algorithms::sql_variable_parser::EndpointInfo,
    routes::custom_endpoints::endpoint_crud::CreateEndpointMethod,
};
use anyhow::Result;
use sqlx::{FromRow, PgPool, Postgres};

struct DbEndpoint {
    pub path: String,
    pub method: String,
    pub handler_info_json: String,
    pub allowed_groups_json: String,
}

fn parse_endpoints_vec(req: CreateEndpointRequest) -> Result<DbEndpoint> {
    let parsed_endpoints_result: anyhow::Result<Vec<EndpointInfo>> = req
        .endpoints_info
        .into_iter()
        .map(EndpointInfo::from_request)
        .collect();

    let parsed_endpoints = parsed_endpoints_result?;

    let parsed_endpoints_json_text = serde_json::to_string(&parsed_endpoints)?;

    let allowed_groups_json = serde_json::to_string(&req.allowed_groups)?;

    Ok(DbEndpoint {
        path: req.path.clone(),
        method: req.method.to_string().to_string(),
        handler_info_json: parsed_endpoints_json_text,
        allowed_groups_json,
    })
}

pub async fn create_endpoint(db_pool: &PgPool, req: CreateEndpointRequest) -> Result<()> {
    let db_endpoint = parse_endpoints_vec(req)?;

    sqlx::query(
        r#"
            INSERT INTO __B_endpoints 
            (req_path, req_method, handler_info, allowed_groups)
            VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(db_endpoint.path)
    .bind(db_endpoint.method)
    .bind(db_endpoint.handler_info_json)
    .bind(db_endpoint.allowed_groups_json)
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn get_endpoints(db_pool: &PgPool) -> Result<Vec<CreateEndpointRequest>> {
    #[derive(FromRow)]
    struct DbReadEndpoint {
        pub req_path: String,
        pub req_method: String,
        pub handler_info: String,
        pub allowed_groups: String,
    }

    fn to_create_request(db_read: DbReadEndpoint) -> Result<CreateEndpointRequest> {
        let req = CreateEndpointRequest {
            path: db_read.req_path,
            method: CreateEndpointMethod::from_str(&db_read.req_method)?,
            allowed_groups: serde_json::from_str(&db_read.allowed_groups)?,
            endpoints_info: serde_json::from_str::<Vec<EndpointInfo>>(&db_read.handler_info)?
                .into_iter()
                .map(EndpointInfo::to_request)
                .collect(),
        };
        Ok(req)
    }

    let endpoints = sqlx::query_as::<Postgres, DbReadEndpoint>(
        r#"
            SELECT req_path, req_method, handler_info, allowed_groups
            FROM __B_endpoints
        "#,
    )
    .fetch_all(db_pool)
    .await?;

    Ok(endpoints
        .into_iter()
        .map(to_create_request)
        .collect::<Result<Vec<CreateEndpointRequest>>>()?)
}

pub async fn update_endpoint(
    db_pool: &PgPool,
    endpoint_id: i32,
    req: CreateEndpointRequest,
) -> Result<()> {
    let db_endpoint = parse_endpoints_vec(req)?;

    sqlx::query(
        r#"
            UPDATE __B_endpoints 
            SET req_path=$1, req_method=$2, handler_info=$3, allowed_groups=$4
            where id=$5::int
        "#,
    )
    .bind(db_endpoint.path)
    .bind(db_endpoint.method)
    .bind(db_endpoint.handler_info_json)
    .bind(db_endpoint.allowed_groups_json)
    .bind(endpoint_id)
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn delete_endpoint(db_pool: &PgPool, endpoint_id: i32) -> Result<()> {
    sqlx::query("DELETE FROM __B_endpoints WHERE id=$1::int")
        .bind(endpoint_id)
        .execute(db_pool)
        .await?;
    Ok(())
}
