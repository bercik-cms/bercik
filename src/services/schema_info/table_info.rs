use crate::services::schema_info::special_column_info::special_column_info;
use crate::types::column_info::ColumnInfo;
use crate::types::column_info::ForeignKeyMap;
use crate::types::table_info::TableInfo;
use anyhow::Context;
use anyhow::Result;
use sqlx::FromRow;
use sqlx::PgPool;
use sqlx::Postgres;

use super::fkey_info::foreign_key_map;
use super::fkey_info::get_all_foreign_keys;

pub async fn get_table_names(db_pool: &PgPool) -> Result<Vec<String>> {
    #[derive(FromRow)]
    struct TableName {
        pub name: String,
    }

    let query = r#"
        SELECT table_name as name
        FROM information_schema.tables
        WHERE table_type = 'BASE TABLE'
        AND table_schema NOT IN ('pg_catalog', 'information_schema');
    "#;

    Ok(sqlx::query_as::<Postgres, TableName>(query)
        .fetch_all(db_pool)
        .await?
        .into_iter()
        .map(|x| x.name)
        .collect())
}

pub async fn get_table_columns<'a>(db_pool: &PgPool, table_name: &str) -> Result<Vec<ColumnInfo>> {
    let query = r#"
    SELECT column_name as name, data_type, is_nullable::bool, coalesce(column_default, '') as column_default
        FROM information_schema.columns
        WHERE table_schema = 'public'
        AND table_name = $1;
    "#;

    Ok(sqlx::query_as::<Postgres, ColumnInfo>(query)
        .bind(table_name)
        .fetch_all(db_pool)
        .await?)
}

pub async fn get_table_info(db_pool: &PgPool) -> Result<Vec<TableInfo>> {
    let mut result = Vec::new();

    let table_names = get_table_names(db_pool).await.context("get_table_names")?;
    let special_column_info = special_column_info(db_pool).await?;

    for table_name in table_names {
        let columns = get_table_columns(db_pool, &table_name).await?;
        result.push(TableInfo {
            columns,
            external_references: special_column_info
                .get_external_refs(&table_name)
                .unwrap_or(&vec![])
                .to_vec(),
            table_name,
        })
    }

    Ok(result)
}
