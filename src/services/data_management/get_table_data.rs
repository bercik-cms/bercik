use crate::services::schema_info::table_info::get_table_info;
use crate::types::arbitrary_sql_row::ArbitrarySqlRow;
use crate::types::table_info::TableInfo;
use anyhow::anyhow;
use anyhow::Result;
use sqlx::PgPool;
use sqlx::Postgres;

fn build_query(table_info: TableInfo) -> String {
    let mut query = String::new();

    query.push_str("SELECT ");

    for col in table_info.columns.iter() {
        query.push_str(&format!("{}::varchar, ", col.name));
    }

    query.pop();
    query.pop();
    query.push(' ');

    query.push_str(&format!("FROM {}", table_info.table_name));
    query
}

pub async fn get_table_data(db_pool: &PgPool, table_name: &str) -> Result<Vec<ArbitrarySqlRow>> {
    let table_info = get_table_info(db_pool)
        .await?
        .into_iter()
        .find(|x| x.table_name == table_name)
        .ok_or(anyhow!("Table name not found"))?;

    let query = build_query(table_info);
    println!("query: {}", query);

    Ok(sqlx::query_as::<Postgres, ArbitrarySqlRow>(&query)
        .fetch_all(db_pool)
        .await?)
}
