use anyhow::Result;
use sqlx::PgPool;

pub async fn init_tables(db_pool: &PgPool) -> Result<()> {
    let query = include_str!("./init_endpoints.sql");
    sqlx::query(query).execute(db_pool).await?;

    Ok(())
}
