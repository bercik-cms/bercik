use super::super::schema_info::table_info::get_table_info;
use crate::routes::data_management::insert_data::{ColumnValue, InsertDataRequest};
use anyhow::{anyhow, Result};
use sqlx::PgPool;

pub async fn build_insert_query(
    db_pool: &PgPool,
    insert_data_request: &InsertDataRequest,
) -> Result<String> {
    let table_name = &insert_data_request.table_name;
    let tables_info = get_table_info(db_pool).await?;

    let mut names: Vec<String> = Vec::new();
    let mut values: Vec<String> = Vec::new();

    let table_info = tables_info
        .iter()
        .find(|it| &it.table_name == table_name)
        .ok_or(anyhow!("Table of that name not found"))?;

    for (info, insert_val) in table_info
        .columns
        .iter()
        .zip(insert_data_request.values.iter())
    {
        let name = &info.name;
        let value = match insert_val {
            ColumnValue {
                value: _,
                use_null: true,
                use_default: false,
            } => "null".to_string(),
            ColumnValue {
                value: _,
                use_null: _,
                use_default: true,
            } => "default".to_string(),
            ColumnValue {
                value: val,
                use_null: false,
                use_default: false,
            } => format!("'{}'", val),
        };
        let col_type = &info.data_type;

        names.push(name.to_string());
        values.push(if value != "null" && value != "default" {
            format!("{}::{}", value, col_type)
        } else {
            value
        });
    }

    Ok(format!(
        "INSERT INTO {} ({}) VALUES ({})",
        table_name,
        names.join(", "),
        values.join(", ")
    ))
}

pub async fn insert_data(db_pool: &PgPool, data: InsertDataRequest) -> Result<()> {
    let query = build_insert_query(db_pool, &data).await?;
    println!("{}", query);
    sqlx::query(&query).execute(db_pool).await?;
    Ok(())
}
