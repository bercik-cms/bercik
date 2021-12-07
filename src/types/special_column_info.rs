use serde::{Deserialize, Serialize};
use sqlx::Type;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum SpecialColumnType {
    PrimaryKey,
    ForeignKey {
        references_table: String,
        references_column: String,
    },
}

#[derive(Debug)]
pub struct SpecialColumnInfo {
    pub table_name: String,
    pub column_name: String,
    pub special_type: SpecialColumnType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExternalReferenceInfo {
    pub table_name: String,
    pub column_name: String,
    pub references_column: String,
}

impl SpecialColumnInfo {
    pub fn special_column_map_key(&self) -> SpecialColumnMapKey {
        SpecialColumnMapKey {
            table_name: self.table_name.clone(),
            column_name: self.column_name.clone(),
        }
    }

    pub fn external_ref_map_key_val(&self) -> Option<(String, ExternalReferenceInfo)> {
        if let SpecialColumnType::ForeignKey {
            ref references_table,
            ref references_column,
        } = self.special_type
        {
            Some((
                references_table.clone(),
                ExternalReferenceInfo {
                    table_name: self.table_name.clone(),
                    column_name: self.column_name.clone(),
                    references_column: references_column.clone(),
                },
            ))
        } else {
            None
        }
    }

    pub fn get_type(&self) -> SpecialColumnType {
        self.special_type.clone()
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct SpecialColumnMapKey {
    pub table_name: String,
    pub column_name: String,
}

#[derive(Debug)]
pub struct SpecialColumnMap {
    map: HashMap<SpecialColumnMapKey, SpecialColumnInfo>,
    external_ref_map: HashMap<String, Vec<ExternalReferenceInfo>>,
}

impl SpecialColumnMap {
    pub fn build(cols: Vec<SpecialColumnInfo>) -> Self {
        let mut map = HashMap::new();
        let mut external_ref_map = HashMap::<String, Vec<ExternalReferenceInfo>>::new();

        for col in cols {
            if let Some((key, val)) = col.external_ref_map_key_val() {
                if external_ref_map.contains_key(&key) {
                    external_ref_map.get_mut(&key).unwrap().push(val);
                } else {
                    external_ref_map.insert(key, vec![val]);
                }
            }
            map.insert(col.special_column_map_key(), col);
        }

        Self {
            map,
            external_ref_map,
        }
    }

    pub fn get_column_special_info(&self, key: &SpecialColumnMapKey) -> Option<&SpecialColumnInfo> {
        self.map.get(key)
    }

    pub fn get_column_special_info_type(
        &self,
        key: &SpecialColumnMapKey,
    ) -> Option<SpecialColumnType> {
        self.get_column_special_info(key)
            .map(|x| x.special_type.clone())
    }

    pub fn get_external_refs(&self, table_name: &str) -> Option<&Vec<ExternalReferenceInfo>> {
        self.external_ref_map.get(table_name)
    }
}
