use axum::{
    async_trait,
    extract::{FromRequest, RequestParts, TypedHeader},
    http::StatusCode,
};
use headers::{authorization::Bearer, Authorization};
use jsonwebtoken::{decode, Validation};
use login_service::KEYS;
use serde::{Deserialize, Serialize};

pub mod change_password;
pub mod create_users;
pub mod create_users_route;
pub mod create_users_service;
pub mod login_route;
pub mod login_service;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    username: String,
    user_group: String,
    exp: usize,
}

impl Claims {
    pub fn must_be_admin(&self) -> Result<(), (StatusCode, String)> {
        if self.user_group == "ADMIN" {
            Ok(())
        } else {
            Err((
                StatusCode::UNAUTHORIZED,
                "User must be an admin".to_string(),
            ))
        }
    }
}

#[async_trait]
impl<B> FromRequest<B> for Claims
where
    B: Send,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request(req)
                .await
                .map_err(|_| {
                    (
                        StatusCode::UNAUTHORIZED,
                        "Bad authorization header".to_string(),
                    )
                })?;

        // Decode the user data
        let token_data = decode::<Claims>(bearer.token(), &KEYS.decoding, &Validation::default())
            .map_err(|it| {
            (
                StatusCode::UNAUTHORIZED,
                format!("Could not decode token: {}", it),
            )
        })?;

        Ok(token_data.claims)
    }
}
