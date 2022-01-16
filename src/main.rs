use anyhow::Context;
use axum::{
    extract::Extension,
    http::StatusCode,
    routing::{any, get, post},
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
mod auth;
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

    let app = Router::new()
        .route(
            "/endpoint/*path",
            any(routes::custom_endpoints::custom_endpoint),
        )
        .route(
            "/api/create-endpoint",
            post(routes::custom_endpoints::endpoint_crud::create_endpoint),
        )
        .route(
            "/api/test-endpoint",
            post(routes::custom_endpoints::endpoint_test::endpoint_test),
        )
        .route(
            "/api/get-endpoints",
            get(routes::custom_endpoints::endpoint_crud::get_endpoints),
        )
        .route(
            "/api/update-endpoint",
            post(routes::custom_endpoints::endpoint_crud::update_endpoint),
        )
        .route(
            "/api/delete-endpoint",
            post(routes::custom_endpoints::endpoint_crud::delete_endpoint),
        )
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
        .route("/api/login", post(auth::login_route::login))
        .route(
            "/api/change-password",
            post(auth::change_password::change_password),
        )
        .route(
            "/api/create-users",
            post(auth::create_users_route::create_users),
        )
        .route(
            "/api/delete-user",
            post(auth::delete_user_route::delete_user_route),
        )
        .route(
            "/api/users-info",
            post(auth::get_users_route::get_users_route),
        )
        .layer(AddExtensionLayer::new(db_pool));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
