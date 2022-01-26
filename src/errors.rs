
use actix_web::{error::ResponseError, HttpResponse};
// use actix_web::error::InternalError;

use derive_more::Display;

#[derive(Debug, Display)]
pub enum ServiceError {
    #[display(fmt = "Internal Server Error: {}", _0)]
    InternalServerError(String),

    #[display(fmt = "Not Found")]
    NotFound,

    #[display(fmt = "Invalid credentials")]
    Unauthorized,

    #[display(fmt = "Forbidden: {}"; _0)]
    Forbidden(String),
}

// impl ResponseError trait allows to convert our errors into http responses with appropriate data
impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ServiceError::InternalServerError(ref message) => {
                HttpResponse::InternalServerError().json(message)
            }
            ServiceError::NotFound => {
                HttpResponse::NotFound().json("Pokemon not found.")
            }
            ServiceError::Unauthorized => {
                HttpResponse::Unauthorized().json("Invalid credentials provided.")
            }
            ServiceError::Forbidden(ref message) => {
                HttpResponse::Forbidden().json(message)
            }
        }
    }
}
