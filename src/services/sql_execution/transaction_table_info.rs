// Only used to get mermaid.js ER diagram diff, issues with type system

use crate::types::{
    column_info::{ColumnInfo, ColumnInfoWithSpecial},
    special_column_info::{
        SpecialColumnInfo, SpecialColumnMap, SpecialColumnMapKey, SpecialColumnType,
    },
    table_info::{ForeignKeyInfo, TableInfo},
};

use anyhow::Context;
use anyhow::Result;
use sqlx::FromRow;
use sqlx::{Postgres, Transaction};

pub async fn get_table_info<'a>(db_pool: &mut Transaction<'a, Postgres>) -> Result<Vec<TableInfo>> {
    let mut result = Vec::new();

    let table_names = get_table_names(db_pool).await.context("get_table_names")?;
    let special_column_info = special_column_info(db_pool).await?;

    for table_name in table_names {
        let columns =
            get_table_columns_with_special(db_pool, &table_name, &special_column_info).await?;

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

async fn get_table_columns_with_special<'a>(
    db_pool: &mut Transaction<'a, Postgres>,
    table_name: &str,
    special_info: &SpecialColumnMap,
) -> Result<Vec<ColumnInfoWithSpecial>> {
    let columns = get_table_columns(db_pool, table_name).await?;
    let mut result = Vec::new();

    for column in columns {
        let column_name = column.name.to_string();
        let table_name = table_name.to_string();
        let with_special = column.with_special(special_info.get_column_special_info_type(
            &SpecialColumnMapKey {
                table_name,
                column_name,
            },
        ));
        result.push(with_special);
    }

    Ok(result)
}

async fn get_table_columns<'a>(
    db_pool: &mut Transaction<'a, Postgres>,
    table_name: &str,
) -> Result<Vec<ColumnInfo>> {
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

async fn get_table_names<'a>(db_pool: &mut Transaction<'a, Postgres>) -> Result<Vec<String>> {
    #[derive(FromRow)]
    struct TableName {
        pub name: String,
    }

    let query = r#"
        SELECT table_name as name
        FROM information_schema.tables
        WHERE table_type = 'BASE TABLE'
        AND table_schema NOT IN ('pg_catalog', 'information_schema')
        AND table_name NOT LIKE '__b_%';
    "#;

    Ok(sqlx::query_as::<Postgres, TableName>(query)
        .fetch_all(db_pool)
        .await?
        .into_iter()
        .map(|x| x.name)
        .collect())
}

async fn special_column_info<'a>(
    db_pool: &mut Transaction<'a, Postgres>,
) -> Result<SpecialColumnMap> {
    let fkey_info = special_column_info_fkeys(db_pool).await?;
    let pkey_info = special_column_info_pkeys(db_pool).await?;

    Ok(SpecialColumnMap::build(
        fkey_info.into_iter().chain(pkey_info.into_iter()).collect(),
    ))
}

async fn special_column_info_fkeys<'a>(
    db_pool: &mut Transaction<'a, Postgres>,
) -> Result<Vec<SpecialColumnInfo>> {
    fn to_special_column_info(intermediate: ForeignKeyInfo) -> SpecialColumnInfo {
        SpecialColumnInfo {
            table_name: intermediate.source_table,
            column_name: intermediate.source_column,
            special_type: SpecialColumnType::ForeignKey {
                references_table: intermediate.target_table,
                references_column: intermediate.target_column,
            },
        }
    }

    Ok(get_all_foreign_keys(db_pool)
        .await?
        .into_iter()
        .map(to_special_column_info)
        .collect())
}

pub async fn get_all_foreign_keys<'a>(
    db_pool: &mut Transaction<'a, Postgres>,
) -> Result<Vec<ForeignKeyInfo>> {
    // https://stackoverflow.com/a/25925751
    let query = r#"
    SELECT source_table::regclass::text, source_attr.attname AS source_column,
           target_table::regclass::text, target_attr.attname AS target_column
    FROM pg_attribute target_attr, pg_attribute source_attr,
    (SELECT source_table, target_table, source_constraints[i] source_constraints, target_constraints[i] AS target_constraints
    FROM
        (SELECT conrelid as source_table, confrelid AS target_table, conkey AS source_constraints, confkey AS target_constraints,
        generate_series(1, array_upper(conkey, 1)) AS i
        FROM pg_constraint
        WHERE contype = 'f'
        ) query1
    ) query2
    WHERE target_attr.attnum = target_constraints AND target_attr.attrelid = target_table AND
          source_attr.attnum = source_constraints AND source_attr.attrelid = source_table
    "#;

    Ok(sqlx::query_as::<Postgres, ForeignKeyInfo>(query)
        .fetch_all(db_pool)
        .await?)
}

async fn special_column_info_pkeys<'a>(
    db_pool: &mut Transaction<'a, Postgres>,
) -> Result<Vec<SpecialColumnInfo>> {
    #[derive(FromRow)]
    struct PKeyColumn {
        table_name: String,
        column_name: String,
        _data_type: String, // TODO: delete if unnecessary
    }

    fn to_special_column_info(intermediate: PKeyColumn) -> SpecialColumnInfo {
        SpecialColumnInfo {
            table_name: intermediate.table_name,
            column_name: intermediate.column_name,
            special_type: SpecialColumnType::PrimaryKey,
        }
    }

    // https://wiki.postgresql.org/wiki/Retrieve_primary_key_columns
    let query = r#"
    SELECT               
        pg_class.relname as table_name,
        pg_attribute.attname as column_name, 
        format_type(pg_attribute.atttypid, pg_attribute.atttypmod) as _data_type 
    FROM pg_index, pg_class, pg_attribute, pg_namespace 
    WHERE 
        indrelid = pg_class.oid AND 
        nspname = 'public' AND 
        pg_class.relnamespace = pg_namespace.oid AND 
        pg_attribute.attrelid = pg_class.oid AND 
        pg_attribute.attnum = any(pg_index.indkey)
        AND indisprimary
    "#;

    Ok(sqlx::query_as::<Postgres, PKeyColumn>(query)
        .fetch_all(db_pool)
        .await?
        .into_iter()
        .map(to_special_column_info)
        .collect())
}
