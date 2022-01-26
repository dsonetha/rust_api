
use crate::models::{User, UserData};

use crate::errors::ServiceError;
use jwt_simple::prelude::*;
use bcrypt::{DEFAULT_COST, hash, verify, BcryptError};

lazy_static! {
    static ref SECRET: String = std::env::var("API_SECRET").expect("API_SECRET env variable must be set");
    static ref SECRET_KEY: HS256Key = HS256Key::from_bytes(SECRET.as_bytes());
}

pub fn encode_password(pwd: &str) -> Result<String, BcryptError> {
    hash(pwd, DEFAULT_COST)
}

pub fn verify_password(pwd: &str, hashed: &str) -> Result<bool, BcryptError> {
    verify(pwd, hashed)
}

pub fn generate_token(user: User) -> Result<String, ServiceError> {
    // create claims valid for 2 hours
    let user_data = UserData {
        id: user.id,
        email: user.email,
        first_name: user.first_name,
        last_name: user.last_name,
        created_at: user.created_at
    };
    let claims = Claims::with_custom_claims(user_data, Duration::from_hours(2));
    let token = match SECRET_KEY.authenticate(claims) {
        Ok(token) => token,
        Err(e) => return Err(ServiceError::InternalServerError(e.to_string()))
    };
    Ok(token)
}

pub fn validate_token(auth_token: &str) -> Result<UserData, ServiceError> {
    match SECRET_KEY.verify_token::<UserData>(&auth_token, None) {
        Ok(claims) => Ok(claims.custom),
        Err(_) => Err(ServiceError::Unauthorized),
    }
}