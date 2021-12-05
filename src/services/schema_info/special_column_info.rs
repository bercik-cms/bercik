use anyhow::Result;
use sqlx::PgPool;

use crate::types::special_column_info::SpecialColumnMap;

use super::{fkey_info::special_column_info_fkeys, pkey_info::special_column_info_pkeys};

pub async fn special_column_info(db_pool: &PgPool) -> Result<SpecialColumnMap> {
    let fkey_info = special_column_info_fkeys(db_pool).await?;
    let pkey_info = special_column_info_pkeys(db_pool).await?;

    Ok(SpecialColumnMap::build(
        fkey_info.into_iter().chain(pkey_info.into_iter()).collect(),
    ))
}
