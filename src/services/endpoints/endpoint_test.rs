use crate::algorithms::{
    endpoint_execution::{EndpointExecutionRuntime, ExecutionResult},
    sql_variable_parser::EndpointInfo,
};
use anyhow::Result;
use sqlx::PgPool;
use std::collections::HashMap;

pub async fn test_endpoint(
    db_pool: &PgPool,
    execution_info: Vec<EndpointInfo>,
    request_variables: HashMap<String, String>,
) -> Result<HashMap<String, ExecutionResult>> {
    let mut runtime = EndpointExecutionRuntime::new(request_variables);

    let mut transaction = db_pool.begin().await?;

    let result = runtime.execute(&mut transaction, &execution_info).await?;

    transaction.rollback().await?;
    Ok(result)
}
