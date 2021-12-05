use crate::types::{
    special_column_info::{SpecialColumnInfo, SpecialColumnType},
    table_info::ForeignKeyInfo,
};
use anyhow::Result;
use sqlx::{PgPool, Postgres};

pub async fn get_foreign_keys_of_table(
    db_pool: &PgPool,
    table_name: &str,
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
          source_attr.attnum = source_constraints AND source_attr.attrelid = source_table AND
          (source_table=$1::regclass OR target_table=$2::regclass);
    "#;

    Ok(sqlx::query_as::<Postgres, ForeignKeyInfo>(query)
        .bind(table_name)
        .bind(table_name)
        .fetch_all(db_pool)
        .await?)
}

pub async fn get_all_foreign_keys(db_pool: &PgPool) -> Result<Vec<ForeignKeyInfo>> {
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

pub async fn special_column_info_fkeys(db_pool: &PgPool) -> Result<Vec<SpecialColumnInfo>> {
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

use std::collections::HashMap;
pub fn foreign_key_map<'a>(
    fkeys: &'a Vec<ForeignKeyInfo>,
) -> HashMap<(&'a str, &'a str), &'a ForeignKeyInfo> {
    let mut map = HashMap::new();
    for fkey in fkeys {
        map.insert(
            (fkey.source_table.as_ref(), fkey.source_column.as_ref()),
            fkey,
        );
    }
    map
}
