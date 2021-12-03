use crate::routes::schema_editing::table_field_types::TableField;
use sqlx::postgres::PgPool;

fn build_field_lines(fields: &Vec<TableField>) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    lines.push("id SERIAL PRIMARY KEY".into());

    for field in fields {
        let name = &field.name;
        let f_type = &field.field_type.to_postgres_type();
        let not_null = if field.not_null { "NOT NULL" } else { "" };
        let default = &field.default.to_postgres_default_value();

        lines.push([name, f_type, not_null, default].join(" "));
    }

    lines
}

fn build_create_sql(name: &str, fields: &Vec<TableField>) -> String {
    let lines = build_field_lines(fields);
    let field_lines = lines.join(", ");

    format!(
        "CREATE TABLE {table_name} ( {field_lines} )",
        table_name = name,
        field_lines = field_lines,
    )
}

pub async fn create_table_from_form(
    table_name: &str,
    fields: &Vec<TableField>,
    db_pool: PgPool,
) -> anyhow::Result<()> {
    let query = build_create_sql(table_name, fields);
    sqlx::query(&query).execute(&db_pool).await?;

    Ok(())
}
