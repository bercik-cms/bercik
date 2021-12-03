use axum::extract::{Extension, Json};
use hyper::StatusCode;
use sqlx::PgPool;

use super::table_field_types::TableField;
use crate::err_utils::to_internal;
use crate::services::schema_editing::form_table_creation::create_table_from_form;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateTableFormRequest {
    pub table_name: String,
    pub table_fields: Vec<TableField>,
}

pub async fn create_table_form(
    Extension(pool): Extension<PgPool>,
    Json(form_request): Json<CreateTableFormRequest>,
) -> Result<(), (StatusCode, String)> {
    create_table_from_form(&form_request.table_name, &form_request.table_fields, pool)
        .await
        .map_err(to_internal)?;

    Ok(())
}
