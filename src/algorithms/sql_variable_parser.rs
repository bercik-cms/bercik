use std::collections::HashMap;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

pub struct SqlWithVariables {
    pub sql: String,
    pub variables: Vec<String>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct EndpointInfoCreateRequest {
    pub name: String,
    pub sql: String,
    pub children: Vec<EndpointInfoCreateRequest>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct EndpointInfo {
    pub name: String,
    pub original_sql: String,
    pub parsed_sql: String,
    pub variables: Vec<String>,
    pub children: Vec<EndpointInfo>,
}

impl EndpointInfo {
    pub fn from_request(req: EndpointInfoCreateRequest) -> Result<Self> {
        let name = req.name;
        let original_sql = req.sql;

        let sql_with_variables = SqlWithVariables::from_sql(&original_sql)?;
        let parsed_sql = sql_with_variables.sql;
        let variables = sql_with_variables.variables;

        let children_result: Result<Vec<EndpointInfo>> = req
            .children
            .into_iter()
            .map(|it| EndpointInfo::from_request(it))
            .collect();

        let children = children_result?;

        Ok(Self {
            name,
            original_sql,
            parsed_sql,
            variables,
            children,
        })
    }

    pub fn to_request(self) -> EndpointInfoCreateRequest {
        EndpointInfoCreateRequest {
            name: self.name,
            sql: self.original_sql,
            children: self.children.into_iter().map(Self::to_request).collect(),
        }
    }
}

impl SqlWithVariables {
    pub fn from_sql(mut sql: &str) -> Result<Self> {
        fn pass_until(mut s: &str, c: char) -> Option<(&str, &str)> {
            let original = s;

            while s.len() != 0 && s.chars().next() != Some(c) {
                s = &s[1..];
            }

            let diff = original.len() - s.len();
            let removed = &original[0..diff];

            if s.chars().next() == Some(c) {
                Some((removed, &s[1..]))
            } else {
                None
            }
        }

        let mut result_sql = String::with_capacity(sql.len());
        let mut counter = (1_usize..).into_iter();
        let mut variables: Vec<String> = Vec::new();

        while sql != "" {
            if sql.len() >= 2 && &sql[0..2] == "${" {
                if let Some((removed, continuation)) = pass_until(sql, '}') {
                    variables.push((&removed[2..]).to_string()); // Remove "${" from beginning
                    result_sql.push_str(&format!("${num}", num = counter.next().unwrap()));
                    sql = continuation;
                    continue;
                } else {
                    return Err(anyhow!("Variable block not closed"));
                }
            }

            result_sql.push(sql.chars().next().unwrap());
            sql = &sql[1..];
        }

        Ok(Self {
            sql: result_sql,
            variables,
        })
    }

    pub fn get_bind_vec<'a>(&self, data: &'a HashMap<String, String>) -> Option<Vec<&'a str>> {
        let mut result = Vec::with_capacity(self.variables.len());

        for variable in &self.variables {
            let value = data.get(variable)?.as_str();
            result.push(value);
        }

        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing_test() {
        let parsed =
            SqlWithVariables::from_sql("SELECT name FROM users WHERE name=${name}").unwrap();
        assert_eq!(&parsed.sql, "SELECT name FROM users WHERE name=$1");
        assert_eq!(&parsed.variables, &vec!["name"]);
    }

    #[test]
    fn parsing_multiple() {
        let sql = "select * from users where name=${name} and age=${age} or name = upper(${name})";
        let parsed = SqlWithVariables::from_sql(sql).unwrap();

        assert_eq!(
            &parsed.sql,
            "select * from users where name=$1 and age=$2 or name = upper($3)"
        );
        assert_eq!(&parsed.variables, &vec!["name", "age", "name"]);
    }

    #[test]
    #[should_panic]
    fn not_closed_panics() {
        SqlWithVariables::from_sql("SELECT ${name").unwrap();
    }

    #[test]
    fn bind_vec() {
        let sql = "select ${name}, ${age}, ${food}, ${name}";

        let mut data = HashMap::new();
        data.insert("name".into(), "Adam".into());
        data.insert("age".into(), "24".into());
        data.insert("food".into(), "bigos".into());

        let result = SqlWithVariables::from_sql(sql)
            .unwrap()
            .get_bind_vec(&data)
            .unwrap();

        assert_eq!(result, vec!["Adam", "24", "bigos", "Adam"]);
    }

    #[test]
    fn parsing_endpoint_info() {
        let req = EndpointInfoCreateRequest {
            name: "user".into(),
            sql: "SELECT id as private_id, username FROM users WHERE id=${req.userId}".into(),
            children: vec![EndpointInfoCreateRequest {
                name: "posts".into(),
                sql: "SELECT title, body FROM posts WHERE user_fk=${super.private_id}".into(),
                children: vec![],
            }],
        };

        let expected = EndpointInfo {
            name: "user".into(),
            original_sql: req.sql.clone(),
            parsed_sql: "SELECT id as private_id, username FROM users WHERE id=$1".into(),
            variables: vec!["req.userId".into()],
            children: vec![EndpointInfo {
                name: "posts".into(),
                original_sql: req.children[0].sql.clone(),
                parsed_sql: "SELECT title, body FROM posts WHERE user_fk=$1".into(),
                variables: vec!["super.private_id".into()],
                children: vec![],
            }],
        };

        assert_eq!(EndpointInfo::from_request(req).unwrap(), expected);
    }
}
