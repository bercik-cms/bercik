use crate::auth::create_users_service::create_user;
use anyhow::Result;
use sqlx::{PgPool, Postgres};

async fn create_initial_admin(db_pool: &PgPool) -> Result<()> {
    Ok(())
}

pub async fn init_tables(db_pool: &PgPool) -> Result<()> {
    let queries = vec![
        include_str!("./init_endpoints.sql"),
        include_str!("./init_users.sql"),
    ];

    for query in queries {
        sqlx::query(query).execute(db_pool).await?;
    }

    // Create initial admin if not exists
    if sqlx::query_as::<Postgres, (i32,)>(
        "select count(*)::int from __B_users where username='admin'",
    )
    .fetch_one(db_pool)
    .await?
        != (1,)
    {
        create_user("admin", "ADMIN", Some("admin1".into()), db_pool).await?;
    }

    Ok(())
}
