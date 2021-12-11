use crate::routes::data_management::get_table_data::{GetTableDataRequest, Sorting, WhereClause};
use crate::services::schema_info::table_info::get_table_info;
use crate::types::arbitrary_sql_array_row::ArbitrarySqlArrayRowsAndNames;
use crate::types::special_column_info::SpecialColumnType;
use crate::types::table_info::TableInfo;
use anyhow::anyhow;
use anyhow::Result;
use sqlx::PgPool;

const PAGE_ROW_COUNT: u32 = 100;

fn build_query(table_info: TableInfo, get_data_request: &GetTableDataRequest) -> String {
    let mut query = String::new();

    query.push_str("SELECT ");

    for col in table_info.columns.iter() {
        query.push_str(&format!("{}::varchar, ", col.name));
    }

    query.pop();
    query.pop();
    query.push(' ');

    query.push_str(&format!("FROM {}", table_info.table_name));

    match &get_data_request.where_clause {
        WhereClause::None => {}
        WhereClause::ColumnEquals { col_name, equals } => {
            query.push_str(&format!(" WHERE {} = {}", col_name, equals));
        }
        WhereClause::Custom(custom) => {
            query.push_str(&format!(" WHERE {}", custom));
        }
    };

    match &get_data_request.sorting {
        Sorting::None => {
            let first_pk = table_info.columns.iter().find(|x| match x.special_info {
                Some(SpecialColumnType::PrimaryKey) => true,
                _ => false,
            });

            if let Some(pk) = first_pk {
                query.push_str(&format!(" ORDER BY {} DESC", pk.name))
            }
        }
        Sorting::ColumnAscending(col) => {
            query.push_str(&format!(" ORDER BY {} ASC", col));
        }
        Sorting::ColumnDescending(col) => {
            query.push_str(&format!(" ORDER BY {} DESC", col));
        }
        Sorting::CustomExpression(expr) => {
            query.push_str(&format!(" ORDER BY {}", expr));
        }
    };

    // Pagination
    if let Some(page) = get_data_request.page {
        let offset = page * PAGE_ROW_COUNT;
        query.push_str(&format!(" OFFSET {} ROWS", offset));
    }
    query.push_str(&format!(" FETCH FIRST {} ROWS ONLY", PAGE_ROW_COUNT));

    query
}

pub async fn get_table_data(
    db_pool: &PgPool,
    get_data_request: &GetTableDataRequest,
) -> Result<ArbitrarySqlArrayRowsAndNames> {
    let table_info = get_table_info(db_pool)
        .await?
        .into_iter()
        .find(|x| x.table_name == get_data_request.table_name)
        .ok_or(anyhow!("Table name not found"))?;

    let query = build_query(table_info, get_data_request);
    println!("query: {}", query);

    let rows = sqlx::query(&query).fetch_all(db_pool).await?;

    Ok(ArbitrarySqlArrayRowsAndNames::from_row_vec(rows)?)
}
