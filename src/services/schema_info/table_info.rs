use crate::services::schema_info::fkey_info::get_foreign_keys_of_table;
use crate::types::intermediate_column_info::ForeignKeyMap;
use crate::types::table_field_types::TableField;
use crate::types::table_info::ForeignKeyInfo;
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

pub async fn get_table_fields<'a>(
    db_pool: &PgPool,
    table_name: &str,
    foreign_key_map: &ForeignKeyMap<'a>,
) -> Result<Vec<TableField>> {
    use crate::types::intermediate_column_info::IntermediateColumnInfo;

    let query = r#"
    SELECT column_name as name, data_type, is_nullable::bool, coalesce(column_default, '') as column_default
        FROM information_schema.columns
        WHERE table_schema = 'public'
        AND table_name = $1;
    "#;

    Ok(sqlx::query_as::<Postgres, IntermediateColumnInfo>(query)
        .bind(table_name)
        .fetch_all(db_pool)
        .await?
        .into_iter()
        .map(|x| x.to_table_field(table_name, foreign_key_map))
        .collect::<Vec<_>>())
}

pub async fn get_table_info(db_pool: &PgPool) -> Result<Vec<TableInfo>> {
    let mut result = Vec::new();

    let table_names = get_table_names(db_pool).await.context("get_table_names")?;
    let foreign_keys = get_all_foreign_keys(db_pool)
        .await
        .context("get_all_foreign_keys")?;
    let outbound_foreign_keys = foreign_key_map(&foreign_keys);

    for table_name in &table_names {
        let table_fields = get_table_fields(db_pool, &table_name, &outbound_foreign_keys)
            .await
            .context("get_table_fields")?;

        result.push(TableInfo {
            table_name: table_name.clone(),
            fields: table_fields,
            external_references: foreign_keys
                .iter()
                .filter(|x| &x.target_column == table_name)
                .cloned()
                .collect(),
        });
    }

    Ok(result)
}
