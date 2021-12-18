use crate::{
    algorithms::sql_variable_parser::EndpointInfo, types::arbitrary_sql_row::ArbitrarySqlRow,
};
use anyhow::{anyhow, Result};
use async_recursion::async_recursion;
use serde::Serialize;
use sqlx::{PgPool, Postgres, Transaction};
use std::collections::HashMap;

#[derive(Serialize, Debug)]
pub struct ExecutionResult {
    pub data: HashMap<String, String>,
    pub children: HashMap<String, ExecutionResult>,
}

#[derive(Debug)]
pub struct EndpointExecutionRuntime {
    request_map: HashMap<String, String>,
    execution_maps: Vec<HashMap<String, String>>,
}

impl EndpointExecutionRuntime {
    pub fn new(request_variables: HashMap<String, String>) -> Self {
        Self {
            request_map: request_variables,
            execution_maps: vec![],
        }
    }

    fn push_execution_map(&mut self, map: HashMap<String, String>) {
        self.execution_maps.push(map);
    }

    fn pop_execution_map(&mut self) -> Option<HashMap<String, String>> {
        self.execution_maps.pop()
    }

    fn get_variable_clone(&self, key: &str) -> Result<String> {
        if key.len() >= 4 && &key[0..4] == "req." {
            let key = &key[4..];
            return self
                .request_map
                .get(key)
                .map(|it| it.clone())
                .ok_or(anyhow!("Request key {} not found", key));
        } else if key.len() >= 6 && &key[0..6] == "super." {
            let mut counter = 0_usize;
            let mut inner_key = key;

            while inner_key.len() >= 6 && &inner_key[0..6] == "super." {
                inner_key = &inner_key[6..];
                counter += 1;
            }

            if counter > self.execution_maps.len() {
                return Err(anyhow!(
                    "Too many 'super.'s, reached negative index ({})",
                    key
                ));
            }

            let vec_index = self.execution_maps.len() - counter;
            let map = &self.execution_maps[vec_index];
            return map
                .get(inner_key)
                .map(|it| it.clone())
                .ok_or(anyhow!("Execution key {} not found", key));
        } else {
            return Err(anyhow!(
                "Bad variable name ({}). Should begin with super. or req.",
                key
            ));
        }
    }

    #[async_recursion]
    pub async fn execute<'a>(
        &mut self,
        transaction: &mut Transaction<'a, Postgres>,
        endpoint_infos: &Vec<EndpointInfo>,
    ) -> Result<HashMap<String, ExecutionResult>> {
        let mut final_results = HashMap::<String, ExecutionResult>::new();

        for query in endpoint_infos {
            let mut exec = sqlx::query_as::<Postgres, ArbitrarySqlRow>(&query.parsed_sql);
            for var_name in &query.variables {
                let val = self.get_variable_clone(var_name)?;
                exec = exec.bind(val);
            }
            let results = exec
                .fetch_all(&mut *transaction)
                .await?
                .into_iter()
                .map(|it| it.into_map())
                .collect::<Vec<_>>();

            for result in results.into_iter() {
                self.push_execution_map(result);

                let children_results = self.execute(transaction, &query.children).await?;

                let mut result_map = self
                    .pop_execution_map()
                    .ok_or(anyhow!("Could not pop execution map"))?;

                // delete private fields
                result_map.retain(|key, _value| key.len() < 8 || &key[0..8] != "private_");

                final_results.insert(
                    query.name.clone(),
                    ExecutionResult {
                        data: result_map,
                        children: children_results,
                    },
                );
            }
        }

        Ok(final_results)
    }
}
