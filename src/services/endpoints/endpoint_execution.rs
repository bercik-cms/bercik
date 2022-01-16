use crate::{
    algorithms::{
        endpoint_execution::{EndpointExecutionRuntime, ExecutionResult},
        sql_variable_parser::EndpointInfo,
    },
    routes::custom_endpoints::EndpointExecutionInfo,
};
use anyhow::Result;
use sqlx::PgPool;
use std::collections::HashMap;

pub async fn execute_endpoint(
    db_pool: &PgPool,
    execution_info: EndpointExecutionInfo,
    request_variables: HashMap<String, String>,
) -> Result<HashMap<String, Vec<ExecutionResult>>> {
    let mut runtime = EndpointExecutionRuntime::new(request_variables);
    let endpoint_info_vec =
        serde_json::from_str::<Vec<EndpointInfo>>(&execution_info.handler_info)?;
    let mut transaction = db_pool.begin().await?;

    let result = runtime
        .execute(&mut transaction, &endpoint_info_vec)
        .await?;

    transaction.commit().await?;
    Ok(result)
}
