use anyhow::{Context, Result};
use axum::{
    async_trait,
    extract::{Extension, FromRequest, RequestParts},
    http::StatusCode,
    routing::get,
    AddExtensionLayer, Router,
};
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::env;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv()?;

    // Set the RUST_LOG, if it hasn't been explicitly defined
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "bercik_server=debug")
    }

    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Setup connection pool
    let database_url = env::var("DATABASE_URL").context("Database url not in .env file")?;
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // Make sure database is working
    let row: (i64,) = sqlx::query_as("SELECT $1")
        .bind(150_i64)
        .fetch_one(&db_pool)
        .await?;
    assert_eq!(row.0, 150);

    // build our application with some routes
    let app = Router::new()
        .route("/", get(endpoint_test))
        .layer(AddExtensionLayer::new(db_pool));

    // run it with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn endpoint_test(Extension(pool): Extension<PgPool>) -> std::result::Result<String, String> {
    let row: (i64,) = sqlx::query_as("SELECT $1 + $1")
        .bind(150_i64)
        .fetch_one(&pool)
        .await
        .map_err(|x| format!("{:?}", x))?;

    Ok(format!("{}", row.0))
}
