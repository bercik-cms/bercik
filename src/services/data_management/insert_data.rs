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

    let mut counter = (1..).into_iter();

    for (info, insert_val) in table_info
        .columns
        .iter()
        .zip(insert_data_request.values.iter())
    {
        let name = &info.name;
        let col_type = &info.data_type;

        names.push(name.to_string());
        if insert_val.use_default {
            values.push("default".into());
        } else if insert_val.use_null {
            values.push("null".into())
        } else {
            values.push(format!("${}::{}", counter.next().unwrap(), col_type))
        }
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

    let mut q = sqlx::query(&query);

    for i in data.values {
        if !i.use_null && !i.use_default {
            q = q.bind(i.value);
        }
    }

    q.execute(db_pool).await?;

    Ok(())
}
