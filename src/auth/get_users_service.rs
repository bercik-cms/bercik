use anyhow::Result;
use serde::Serialize;
use sqlx::{FromRow, PgPool, Postgres};

#[derive(Serialize, FromRow)]
pub struct UserInfo {
    pub username: String,
    pub user_group: String,
}

pub async fn get_users_service(db_pool: &PgPool) -> Result<Vec<UserInfo>> {
    Ok(
        sqlx::query_as::<Postgres, UserInfo>("SELECT username, user_group from __B_users")
            .fetch_all(db_pool)
            .await?,
    )
}
