use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;

use super::{
    special_column_info::SpecialColumnType,
    table_field_types::{DefaultValue, TableField, TableFieldType},
    table_info::ForeignKeyInfo,
};

pub type ForeignKeyMap<'a> = HashMap<(&'a str, &'a str), &'a ForeignKeyInfo>;

#[derive(FromRow, Debug, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub column_default: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ColumnInfoWithSpecial {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub column_default: String,
    pub special_info: Option<SpecialColumnType>,
}

impl ColumnInfo {
    pub fn get_table_field_type(
        &self,
        table_name: &str,
        foreign_key_map: &ForeignKeyMap,
    ) -> TableFieldType {
        let data_type = self.data_type.as_str();
        let find_nextval = self.column_default.find("nextval");
        let references = foreign_key_map.get(&(table_name, &self.name));

        match (data_type, find_nextval, references) {
            ("integer", None, None) => TableFieldType::Integer,
            ("integer", Some(0), None) => TableFieldType::Serial,
            ("integer", None, Some(other)) => {
                TableFieldType::ForeignKey(other.target_table.to_string())
            }
            ("real", _, None) => TableFieldType::RealNumber,
            ("character varying", _, None) => TableFieldType::String,
            ("text", _, None) => TableFieldType::Text,
            (other, _, _) => TableFieldType::CustomType(other.to_string()),
        }
    }

    pub fn with_special(self, special_info: Option<SpecialColumnType>) -> ColumnInfoWithSpecial {
        let ColumnInfo {
            name,
            data_type,
            is_nullable,
            column_default,
        } = self;

        ColumnInfoWithSpecial {
            name,
            data_type,
            is_nullable,
            column_default,
            special_info,
        }
    }
}
