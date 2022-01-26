
use crate::schema::*;
use serde::{Deserialize, Serialize};
use actix_web::{FromRequest, HttpRequest, dev::Payload};
use futures::future::{ok, err};


#[derive(Debug, Serialize, Deserialize, Queryable)]
pub struct User {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[table_name="users"]
pub struct NewUser<'a> {
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub email: &'a str,
    pub pwd: &'a str,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InputUser {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String
}

// used in jwt claims and as resources output, do not keep password here
#[derive(Debug, Serialize, Deserialize, Queryable, Clone)]
pub struct UserData {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub created_at: chrono::NaiveDateTime,
}

impl FromRequest for UserData {
    type Error = actix_web::Error;
    type Future = futures::future::Ready<Result<Self, Self::Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        match req.extensions().get::<UserData>() {
            Some(user_data) => return ok(user_data.to_owned()),
            None => return err(actix_web::error::ErrorBadRequest("Could not retrive user data from claims"))
        };
    }

}

#[derive(Deserialize, Clone)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub token: String,
}
