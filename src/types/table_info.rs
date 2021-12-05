use super::table_field_types::TableField;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ForeignKeyInfo {
    pub source_table: String,
    pub source_column: String,
    pub target_table: String,
    pub target_column: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TableInfo {
    pub table_name: String,
    pub fields: Vec<TableField>,
    pub external_references: Vec<ForeignKeyInfo>,
}
