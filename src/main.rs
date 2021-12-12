use anyhow::Context;
use axum::{
    extract::Extension,
    http::StatusCode,
    routing::{get, post},
    AddExtensionLayer, Router,
};
use dotenv::dotenv;
use routes::schema::schema_editing::create_table_form::create_table_form;
use serde_json::{json, Value};
use sqlx::{postgres::PgPoolOptions, Column, Decode, Row};
use sqlx::{PgPool, Postgres};
use std::env;
use std::net::SocketAddr;

use crate::{
    routes::data_management::get_table_data::GetTableDataRequest,
    services::schema_info::{
        fkey_info::get_all_foreign_keys,
        pkey_info::special_column_info_pkeys,
        special_column_info::special_column_info,
        table_info::{self, get_table_info},
    },
    types::{arbitrary_sql_array_row::ArbitrarySqlArrayRow, arbitrary_sql_row::ArbitrarySqlRow},
};

mod algorithms;
mod db_utils;
mod err_utils;
mod routes;
mod services;
mod setup;
mod types;

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

    setup::setup_internal_tables::init_tables(&db_pool).await?;

    let _ = dbg!(special_column_info(&db_pool).await);
    println!(
        "{}",
        serde_json::to_string_pretty(&get_table_info(&db_pool).await?)?
    );

    println!(
        "{}",
        serde_json::to_string(
            &sqlx::query_as::<Postgres, ArbitrarySqlRow>(
                "select (10 +10)::text as first, (2*2)::text as second"
            )
            .fetch_one(&db_pool)
            .await?
        )?
    );

    use crate::routes::data_management::get_table_data::{
        GetTableDataRequest, Sorting, WhereClause,
    };

    println!(
        "{}",
        serde_json::to_string_pretty(
            &services::data_management::get_table_data::get_table_data(
                &db_pool,
                &GetTableDataRequest {
                    table_name: "siemanko".into(),
                    sorting: Sorting::None,
                    where_clause: WhereClause::None,
                    page: None,
                }
            )
            .await?
        )?
    );

    let app = Router::new()
        .route("/api/create-table", post(create_table_form))
        .route(
            "/api/table-info",
            get(routes::schema::schema_info::table_info::get_table_info),
        )
        .route(
            "/api/insert-data",
            post(routes::data_management::insert_data::insert_data),
        )
        .route(
            "/api/get-table-data",
            post(routes::data_management::get_table_data::get_table_data),
        )
        .route(
            "/api/execute-queries",
            post(routes::data_management::sql_editor::execute_queries),
        )
        .layer(AddExtensionLayer::new(db_pool));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
