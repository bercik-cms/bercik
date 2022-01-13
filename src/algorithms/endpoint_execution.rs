use crate::{
    algorithms::sql_variable_parser::EndpointInfo, types::arbitrary_sql_row::ArbitrarySqlRow,
};
use anyhow::{anyhow, Result};
use async_recursion::async_recursion;
use serde::Serialize;
use sqlx::{Postgres, Transaction};
use std::collections::HashMap;

#[derive(Serialize, Debug, PartialEq)]
pub struct ExecutionResult {
    pub data: HashMap<String, String>,
    pub children: HashMap<String, Vec<ExecutionResult>>,
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
    pub async fn execute(
        &mut self,
        #[cfg(test)] mock_exec_service: &mut ExecutionMockService,
        #[cfg(not(test))] transaction: &mut Transaction<'_, Postgres>,
        endpoint_infos: &Vec<EndpointInfo>,
    ) -> Result<HashMap<String, Vec<ExecutionResult>>> {
        let mut final_results = HashMap::<String, Vec<ExecutionResult>>::new();

        for query in endpoint_infos {
            #[cfg(not(test))]
            let mut exec = sqlx::query_as::<Postgres, ArbitrarySqlRow>(&query.parsed_sql);
            for var_name in &query.variables {
                let val = self.get_variable_clone(var_name)?;

                #[cfg(not(test))]
                {
                    exec = exec.bind(val);
                }
                #[cfg(test)]
                {
                    mock_exec_service.bind(&val);
                }
            }

            #[cfg(test)]
            let results = mock_exec_service.simulate_call(&query.parsed_sql);

            #[cfg(not(test))]
            let results = exec
                .fetch_all(&mut *transaction)
                .await?
                .into_iter()
                .map(|it| it.into_map())
                .collect::<Vec<_>>();

            for result in results.into_iter() {
                self.push_execution_map(result);

                #[cfg(test)]
                let children_results = self.execute(mock_exec_service, &query.children).await?;
                #[cfg(not(test))]
                let children_results = self.execute(transaction, &query.children).await?;

                let mut result_map = self
                    .pop_execution_map()
                    .ok_or(anyhow!("Could not pop execution map"))?;

                // delete private fields
                result_map.retain(|key, _value| key.len() < 8 || &key[0..8] != "private_");

                if final_results.contains_key(&query.name) {
                    final_results
                        .get_mut(&query.name)
                        .unwrap()
                        .push(ExecutionResult {
                            data: result_map,
                            children: children_results,
                        })
                } else {
                    final_results.insert(
                        query.name.clone(),
                        vec![ExecutionResult {
                            data: result_map,
                            children: children_results,
                        }],
                    );
                }
            }
        }

        Ok(final_results)
    }
}

#[derive(Debug, PartialEq)]
pub struct ExecutionMockService {
    pub bound_params: Vec<String>,
    pub called_queries: Vec<String>,
    pub result_stack: Vec<Vec<HashMap<String, String>>>,
}

impl ExecutionMockService {
    pub fn new(result_stack: Vec<Vec<HashMap<String, String>>>) -> Self {
        Self {
            result_stack,
            called_queries: vec![],
            bound_params: vec![],
        }
    }

    pub fn bind(&mut self, param: impl std::fmt::Display) {
        self.bound_params.push(format!("{}", param));
    }

    pub fn simulate_call(&mut self, query: &str) -> Vec<HashMap<String, String>> {
        self.called_queries.push(query.to_owned());
        self.result_stack.pop().unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use maplit::hashmap;

    #[tokio::test]
    async fn it_works() {
        let mut mock_service = ExecutionMockService::new(vec![vec![hashmap! {
           "test".into() => "test".into()
        }]]);

        let request_variables = hashmap! {};

        let endpoint_infos = vec![EndpointInfo {
            name: "test".into(),
            variables: vec![],
            parsed_sql: "this sql should be executed".into(),
            original_sql: "".into(),
            children: vec![],
        }];

        let mut execution_runtime = EndpointExecutionRuntime::new(request_variables);

        let final_result = execution_runtime
            .execute(&mut mock_service, &endpoint_infos)
            .await
            .unwrap();

        assert_eq!(
            final_result,
            hashmap! {"test".into() => vec![ExecutionResult{
                data: hashmap! {
                    "test".into() => "test".into(),
                },
                children: hashmap! {}
            }]}
        );

        assert_eq!(
            mock_service.called_queries,
            vec!["this sql should be executed"]
        );
        assert_eq!(mock_service.bound_params, Vec::<&str>::new());
    }

    #[tokio::test]
    async fn error_when_cant_find_req_variable() {
        let mut mock_service = ExecutionMockService::new(vec![vec![hashmap! {
           "test".into() => "test".into()
        }]]);

        let request_variables = hashmap! {};

        let endpoint_infos = vec![EndpointInfo {
            name: "test".into(),
            variables: vec!["req.test_key".into()],
            parsed_sql: "".into(),
            original_sql: "".into(),
            children: vec![],
        }];

        let mut execution_runtime = EndpointExecutionRuntime::new(request_variables);

        let result = execution_runtime
            .execute(&mut mock_service, &endpoint_infos)
            .await;

        let error = result.unwrap_err();
        assert_eq!(error.to_string(), "Request key test_key not found");
    }

    #[tokio::test]
    async fn error_when_too_many_supers() {
        let mut mock_service = ExecutionMockService::new(vec![vec![hashmap! {
           "test".into() => "test".into()
        }]]);

        let request_variables = hashmap! {};

        let endpoint_infos = vec![EndpointInfo {
            name: "test".into(),
            variables: vec!["super.test_key".into()],
            parsed_sql: "should not be executed".into(),
            original_sql: "".into(),
            children: vec![],
        }];

        let mut execution_runtime = EndpointExecutionRuntime::new(request_variables);

        let result = execution_runtime
            .execute(&mut mock_service, &endpoint_infos)
            .await;

        let error = result.unwrap_err();
        assert_eq!(
            error.to_string(),
            "Too many 'super.'s, reached negative index (super.test_key)"
        );

        // check that it didn't execute the query
        assert_eq!(mock_service.called_queries, Vec::<&str>::new());
    }

    #[tokio::test]
    async fn error_when_cant_find_super_variable() {
        let mut mock_service = ExecutionMockService::new(vec![vec![hashmap! {
           "test".into() => "test".into()
        }]]);

        let request_variables = hashmap! {};

        let endpoint_infos = vec![EndpointInfo {
            name: "test".into(),
            variables: vec![],
            parsed_sql: "Should be executed".into(),
            original_sql: "".into(),

            children: vec![EndpointInfo {
                name: "test_inner".into(),
                variables: vec!["super.key_that_doesnt_exist".into()],
                parsed_sql: "Should not be executed".into(),
                original_sql: "".into(),
                children: vec![],
            }],
        }];

        let mut execution_runtime = EndpointExecutionRuntime::new(request_variables);

        let result = execution_runtime
            .execute(&mut mock_service, &endpoint_infos)
            .await;

        let error = result.unwrap_err();
        assert_eq!(
            error.to_string(),
            "Execution key super.key_that_doesnt_exist not found"
        );

        // check that it only executed the outer query
        //
        // in runtime the outer query will be reverted through a transaction
        // after the inner query fails
        assert_eq!(mock_service.called_queries, vec!["Should be executed"]);
    }

    #[tokio::test]
    async fn request_variables_work() {
        let mut mock_service = ExecutionMockService::new(vec![vec![hashmap! {
           "test".into() => "test".into()
        }]]);

        let request_variables = hashmap! {
            "age".to_owned() => "41".to_owned()
        };

        let endpoint_infos = vec![EndpointInfo {
            name: "test".into(),
            variables: vec!["req.age".into()],
            parsed_sql: "select $1".into(),
            original_sql: "".into(),
            children: vec![],
        }];

        let mut execution_runtime = EndpointExecutionRuntime::new(request_variables);

        let final_result = execution_runtime
            .execute(&mut mock_service, &endpoint_infos)
            .await
            .unwrap();

        assert_eq!(
            final_result,
            hashmap! {"test".into() => vec![ExecutionResult{
                data: hashmap! {
                    "test".into() => "test".into(),
                },
                children: hashmap! {}
            }]}
        );

        assert_eq!(mock_service.called_queries, vec!["select $1".to_owned()]);
        assert_eq!(mock_service.bound_params, vec!["41".to_owned()]);
    }

    #[tokio::test]
    async fn super_variables_work() {
        let mut mock_service = ExecutionMockService::new(vec![
            vec![hashmap! {
                "inner_test".into() => "child of test 2".into()
            }],
            vec![hashmap! {
                "inner_test".into() => "child of test 1".into()
            }],
            vec![
                hashmap! {
                    "test".into() => "test 1".into()
                },
                hashmap! {
                    "test".into() => "test 2".into()
                },
            ],
        ]);

        let request_variables = hashmap! {};

        let endpoint_infos = vec![EndpointInfo {
            name: "test".into(),
            variables: vec![],
            parsed_sql: "outer sql".into(),
            original_sql: "".into(),

            children: vec![EndpointInfo {
                name: "test_inner".into(),
                variables: vec!["super.test".into()],
                parsed_sql: "inner sql".into(),
                original_sql: "".into(),
                children: vec![],
            }],
        }];

        let mut execution_runtime = EndpointExecutionRuntime::new(request_variables);

        let final_result = execution_runtime
            .execute(&mut mock_service, &endpoint_infos)
            .await
            .unwrap();

        assert_eq!(
            final_result,
            hashmap! {"test".into() => vec![
                ExecutionResult{
                    data: hashmap! {"test".into() => "test 1".into()},
                    children: hashmap! {
                        "test_inner".into() => vec![
                            ExecutionResult {
                                data: hashmap! {"inner_test".into() => "child of test 1".into()},
                                children: hashmap! {},
                            }
                        ]
                    }
                },
                ExecutionResult {
                    data: hashmap! {"test".into() => "test 2".into()},
                    children: hashmap! {
                        "test_inner".into() => vec![
                            ExecutionResult {
                                data: hashmap! {"inner_test".into() => "child of test 2".into()},
                                children: hashmap! {},
                            }
                        ]
                    }
                }
            ]}
        );
    }
}
