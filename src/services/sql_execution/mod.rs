use anyhow::Result;
use sqlx::PgPool;
use sqlx::Postgres;
use sqlx::Transaction;

use crate::algorithms::mermaid_diagram_generation::MermaidDiagram;
use crate::routes::data_management::sql_editor::{ExecuteQueriesRequest, ExecuteQueriesResponse};
use crate::types::arbitrary_sql_array_row::ArbitrarySqlArrayRowsAndNames;

mod transaction_table_info;

async fn get_query_diff<'a>(
    transaction: &mut Transaction<'a, Postgres>,
    diff_query: &str,
) -> Result<ArbitrarySqlArrayRowsAndNames> {
    Ok(ArbitrarySqlArrayRowsAndNames::from_row_vec(
        sqlx::query(diff_query).fetch_all(transaction).await?,
    )?)
}

async fn get_mermaid_diff<'a>(transaction: &mut Transaction<'a, Postgres>) -> Result<String> {
    use transaction_table_info::{get_all_foreign_keys, get_table_info};

    let table_info = get_table_info(transaction).await?;
    let fkey_info = get_all_foreign_keys(transaction).await?;

    Ok(MermaidDiagram::new(&table_info, &fkey_info)?.0)
}

pub async fn execute_queries(
    db_pool: &PgPool,
    req: &ExecuteQueriesRequest,
) -> Result<ExecuteQueriesResponse> {
    let mut query_results = Vec::<ArbitrarySqlArrayRowsAndNames>::new();
    let mut query_diff: Option<Vec<ArbitrarySqlArrayRowsAndNames>> = None;
    let mut mermaid_diff: Option<Vec<String>> = None;

    let mut transaction = db_pool.begin().await?;

    if req.should_diff_mermaid {
        mermaid_diff = Some(vec![get_mermaid_diff(&mut transaction).await?]);
    }

    if req.should_diff_query {
        query_diff = Some(vec![
            get_query_diff(&mut transaction, &req.diff_query).await?,
        ]);
    }

    for query in req.queries.iter() {
        let rows = sqlx::query(query).fetch_all(&mut transaction).await?;
        query_results.push(ArbitrarySqlArrayRowsAndNames::from_row_vec(rows)?);
    }

    if let Some(ref mut qd) = query_diff {
        qd.push(get_query_diff(&mut transaction, &req.diff_query).await?);
    }

    if let Some(ref mut md) = mermaid_diff {
        md.push(get_mermaid_diff(&mut transaction).await?);
    }

    if req.execute {
        transaction.commit().await?;
    }

    Ok(ExecuteQueriesResponse {
        query_results,
        query_diff,
        mermaid_diff,
    })
}
