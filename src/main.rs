use anyhow::Context;
use axum::{
    extract::Extension,
    http::StatusCode,
    routing::{get, post},
    AddExtensionLayer, Router,
};
use dotenv::dotenv;
use routes::schema_editing::create_table_form::create_table_form;
use serde_json::{json, Value};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::env;
use std::net::SocketAddr;

mod db_utils;
mod err_utils;
mod routes;
mod services;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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

    let app = Router::new()
        .route("/", get(endpoint_test))
        .route("/create", get(create_table))
        .route("/json_test", get(json_value))
        .route("/json_test_test", get(test_json_val))
        .route("/api/create-table", post(create_table_form))
        .layer(AddExtensionLayer::new(db_pool));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn endpoint_test(Extension(pool): Extension<PgPool>) -> Result<String, (StatusCode, String)> {
    let row: (i64,) = sqlx::query_as("SELEC $1 + $1")
        .bind(150_i64)
        .fetch_one(&pool)
        .await
        .map_err(err_utils::to_internal)?;

    Ok(format!("{}", row.0))
}

async fn create_table(Extension(pool): Extension<PgPool>) -> String {
    sqlx::query("create table test()")
        .execute(&pool)
        .await
        .unwrap();
    "Ok".into()
}

async fn json_value() -> axum::response::Json<Value> {
    let value = json!({
        "msg": 2 + 2,
        "inner": {
            "msg": "test",
        }
    });
    axum::response::Json(value)
}

async fn test_json_val() -> axum::response::Json<Value> {
    return axum::response::Json(json!(["test", "test2", "test3",]));
}
