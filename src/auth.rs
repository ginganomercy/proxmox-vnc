use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub id: String,
    pub username: String,
    pub role: String,
    pub exp: usize,
}

pub fn validate_jwt(token: &str, secret: &str) -> Option<Claims> {
    let validation = Validation::new(Algorithm::HS256);
    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    ) {
        Ok(token_data) => Some(token_data.claims),
        Err(_) => None,
    }
}
