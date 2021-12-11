use crate::types::table_info::{ForeignKeyInfo, TableInfo};
use anyhow::{anyhow, Result};

#[derive(Debug, PartialEq)]
pub struct MermaidDiagram(pub String);

impl MermaidDiagram {
    pub fn new(tables: &Vec<TableInfo>, fkeys: &Vec<ForeignKeyInfo>) -> Result<Self> {
        let mut result = String::from("erDiagram ");

        for table in tables {
            result.push_str(&format!("{} {{ ", table.table_name));
            for column in &table.columns {
                result.push_str(&format!("{} {} ", column.data_type, column.name));
            }
            result.push_str("} ");
        }

        fn is_fkey_nullable(fkey: &ForeignKeyInfo, tables: &Vec<TableInfo>) -> Result<bool> {
            let table = tables
                .iter()
                .find(|x| x.table_name == fkey.source_table)
                .ok_or(anyhow!(
                    "Found foreign key info of a table that doesn't exist"
                ))?;

            let column = table
                .columns
                .iter()
                .find(|x| x.name == fkey.source_column)
                .ok_or(anyhow!(
                    "Found foreign key info of a column that doesn't exist"
                ))?;

            Ok(column.is_nullable)
        }

        for fkey in fkeys {
            result.push_str(&format!(
                "{target} {is_nullable}--o{{ {source} : \"{src_c} ref. {trg_c}\" ",
                source = fkey.source_table,
                is_nullable = if is_fkey_nullable(fkey, tables)? {
                    "|o"
                } else {
                    "||"
                },
                target = fkey.target_table,
                src_c = fkey.source_column,
                trg_c = fkey.target_column,
            ));
        }

        Ok(Self(result))
    }
}

#[cfg(test)]
mod tests {
    use crate::types::column_info::ColumnInfoWithSpecial;
    use crate::types::special_column_info::SpecialColumnType;

    use super::*;

    #[test]
    fn works() {
        let table_info = vec![
            TableInfo {
                table_name: "table_one".to_string(),
                columns: vec![ColumnInfoWithSpecial {
                    name: "id".to_string(),
                    data_type: "int".to_string(),
                    is_nullable: false,
                    column_default: "".to_string(),
                    special_info: Some(SpecialColumnType::PrimaryKey),
                }],
                external_references: vec![], // not used for mermaid diagram
            },
            TableInfo {
                table_name: "table_two".to_string(),
                columns: vec![
                    ColumnInfoWithSpecial {
                        name: "id".to_string(),
                        data_type: "int".to_string(),
                        is_nullable: false,
                        column_default: "".to_string(),
                        special_info: Some(SpecialColumnType::PrimaryKey),
                    },
                    ColumnInfoWithSpecial {
                        name: "table_one_fk".to_string(),
                        data_type: "int".to_string(),
                        is_nullable: false,
                        column_default: "".to_string(),
                        special_info: Some(SpecialColumnType::ForeignKey {
                            references_table: "table_one".to_string(),
                            references_column: "id".to_string(),
                        }),
                    },
                ],
                external_references: vec![], // not used for mermaid diagram
            },
        ];

        let fkey_info = vec![ForeignKeyInfo {
            source_table: "table_two".to_string(),
            source_column: "table_one_fk".to_string(),
            target_table: "table_one".to_string(),
            target_column: "id".to_string(),
        }];

        let diagram = MermaidDiagram::new(&table_info, &fkey_info).unwrap();

        assert_eq!(
            diagram,
            MermaidDiagram(
                "\
        erDiagram \
        table_one { int id } \
        table_two { int id int table_one_fk } \
        table_one ||--o{ table_two : \"table_one_fk ref. id\" \
        "
                .to_string()
            )
        );
    }
}
