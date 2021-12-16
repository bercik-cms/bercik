use super::create_users_route::CreateUsersRequest;
use anyhow::{anyhow, Result};
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use passwords::PasswordGenerator;
use rand_core::OsRng;
use serde::Serialize;
use sqlx::{PgPool, Postgres};

#[derive(Debug, Serialize)]
pub struct UsernamePass {
    username: String,
    password: String,
}

pub async fn create_users(req: &CreateUsersRequest, db_pool: &PgPool) -> Result<Vec<UsernamePass>> {
    let username: &str = req.username.as_ref();
    let mut counter = (1_usize..).into_iter();

    let mut new_users: Vec<UsernamePass> = Vec::with_capacity(req.amount);

    if req.amount > 1 && username.matches("{}").into_iter().collect::<Vec<_>>().len() != 1 {
        return Err(anyhow!(
            r#"If creating multiple users,
               username must contain one "{}"
               to replace with sequence!"#
        ));
    }

    for _ in 0..req.amount {
        let mut username_test: String;
        loop {
            username_test = username.replace("{}", &format!("{}", counter.next().unwrap()));

            if sqlx::query_as::<Postgres, (i32,)>(
                "select count(*)::int from __B_users where username=$1",
            )
            .bind(&username_test)
            .fetch_one(db_pool)
            .await?
                == (0,)
            {
                break;
            }
        }

        let pass = create_user(&username_test, &req.user_group, None, db_pool).await?;

        new_users.push(UsernamePass {
            username: username_test,
            password: pass,
        });
    }

    Ok(new_users)
}

pub async fn create_user(
    username: &str,
    user_group: &str,
    mut password: Option<String>,
    db_pool: &PgPool,
) -> Result<String> {
    if let None = password {
        password = Some(
            PasswordGenerator::new()
                .length(16)
                .generate_one()
                .map_err(|it| anyhow!("{}", it))?,
        );
    }

    let password = password.unwrap();

    let salt_string = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|it| anyhow!("ERROR hashing password: {}", it))?
        .to_string();

    sqlx::query("INSERT INTO __B_users (username, password_hash, user_group) VALUES ($1, $2, $3)")
        .bind(username)
        .bind(password_hash)
        .bind(user_group)
        .execute(db_pool)
        .await?;

    Ok(password.to_string())
}
