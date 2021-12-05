use std::collections::HashMap;

#[derive(Debug, Clone)]
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

impl SpecialColumnInfo {
    pub fn special_column_map_key(&self) -> SpecialColumnMapKey {
        SpecialColumnMapKey {
            table_name: self.table_name.clone(),
            column_name: self.column_name.clone(),
        }
    }

    pub fn get_type(&self) -> SpecialColumnType {
        self.special_type.clone()
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
struct SpecialColumnMapKey {
    pub table_name: String,
    pub column_name: String,
}

#[derive(Debug)]
pub struct SpecialColumnMap {
    map: HashMap<SpecialColumnMapKey, SpecialColumnInfo>,
}

impl SpecialColumnMap {
    pub fn build(cols: Vec<SpecialColumnInfo>) -> Self {
        let mut map = HashMap::new();
        for col in cols {
            map.insert(col.special_column_map_key(), col);
        }
        Self { map }
    }

    pub fn get(&self, key: &SpecialColumnMapKey) -> Option<&SpecialColumnInfo> {
        self.map.get(key)
    }
}
