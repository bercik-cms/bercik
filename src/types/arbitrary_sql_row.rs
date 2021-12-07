use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, Column, FromRow, Row};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct ArbitrarySqlRow(HashMap<String, String>);

impl FromRow<'_, PgRow> for ArbitrarySqlRow {
    fn from_row(row: &'_ PgRow) -> Result<Self, sqlx::Error> {
        let mut map = HashMap::new();

        for i in 0..row.len() {
            map.insert(row.column(i).name().into(), row.try_get(i)?);
        }

        Ok(Self(map))
    }
}
