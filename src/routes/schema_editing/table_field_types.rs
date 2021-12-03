use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "content")]
pub enum TableFieldType {
    Integer,
    Serial,
    RealNumber,
    String,
    Text,
    Date,
    ForeignKey(String),
    CustomType(String),
}

#[derive(Serialize, Deserialize, Clone)]
pub enum DefaultValue {
    None,
    Value(String),
}

impl DefaultValue {
    pub fn to_postgres_default_value(&self) -> String {
        match self {
            &Self::None => "".to_string(),
            &Self::Value(ref v) => format!("DEFAULT {}", v),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TableField {
    pub name: String,
    pub field_type: TableFieldType,
    pub not_null: bool,
    pub default: DefaultValue,
}

impl TableFieldType {
    pub fn to_postgres_type(&self) -> String {
        String::from(match self {
            &Self::Integer => "int",
            &Self::Serial => "serial",
            &Self::RealNumber => "real",
            &Self::String => "varchar(255)",
            &Self::Text => "text",
            &Self::Date => "date",
            &Self::ForeignKey(ref str) => str,
            &Self::CustomType(ref str) => str,
        })
    }
}
