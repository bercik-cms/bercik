use anyhow::{anyhow, Result};
use sqlx::{PgPool, Postgres};

pub async fn delete_user_service(db_pool: &PgPool, username: &str) -> Result<()> {
    if sqlx::query_as::<Postgres, (i32,)>("select count(*)::int from __B_users where username=$1")
        .bind(&username)
        .fetch_one(db_pool)
        .await?
        == (0,)
    {
        return Err(anyhow!("Trying to delete a user that does not exist"));
    }

    sqlx::query("DELETE FROM __B_users WHERE username=$1")
        .bind(&username)
        .execute(db_pool)
        .await?;

    Ok(())
}
