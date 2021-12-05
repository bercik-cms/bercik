use anyhow::Result;
use sqlx::{FromRow, PgPool, Postgres};

use crate::types::special_column_info::{SpecialColumnInfo, SpecialColumnType};

pub async fn special_column_info_pkeys(db_pool: &PgPool) -> Result<Vec<SpecialColumnInfo>> {
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
