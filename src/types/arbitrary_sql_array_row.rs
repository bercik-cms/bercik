use anyhow::Result;
use serde::Serialize;
use sqlx::{postgres::PgRow, Column, FromRow, Row};

#[derive(Serialize)]
pub struct ArbitrarySqlArrayRowsAndNames {
    pub names: Vec<String>,
    pub rows: Vec<ArbitrarySqlArrayRow>,
}

impl ArbitrarySqlArrayRowsAndNames {
    pub fn from_row_vec(input_rows: Vec<PgRow>) -> Result<Self> {
        let mut names = Vec::new();
        let mut rows = Vec::new();

        let first_row = input_rows.get(0);
        if let None = first_row {
            // Can't get column names from zero length result, return empty;
            return Ok(Self { names, rows });
        }
        let first_row = first_row.unwrap();

        for i in 0..first_row.len() {
            names.push(first_row.try_column(i)?.name().into())
        }

        for row in &input_rows {
            rows.push(ArbitrarySqlArrayRow::from_row(row)?);
        }

        Ok(Self { names, rows })
    }
}

#[derive(Debug, Serialize)]
pub struct ArbitrarySqlArrayRow(Vec<String>);

impl FromRow<'_, PgRow> for ArbitrarySqlArrayRow {
    fn from_row(row: &'_ PgRow) -> Result<Self, sqlx::Error> {
        let mut vec = Vec::new();

        for i in 0..row.len() {
            vec.push(row.try_get(i)?);
        }

        Ok(Self(vec))
    }
}
