
use super::models::{NewUser, User, InputUser, TokenResponse, LoginRequest};
use super::schema::users::dsl::*;
use super::Pool;
use crate::diesel::{QueryDsl,ExpressionMethods,RunQueryDsl};
use crate::auth::{encode_password, generate_token, verify_password};
use crate::models::UserData;
use actix_web::{web, Error, HttpResponse};
use diesel::dsl::{delete, insert_into};
use std::vec::Vec;
use crate::errors::ServiceError;

// Handler for POST /login
pub async fn login(db: web::Data<Pool>, data: web::Json<LoginRequest>) -> Result<HttpResponse, Error> {
    let LoginRequest { password: data_pwd, email: data_email} = data.into_inner();
    Ok(web::block(move || db_get_user_by_email(db, &data_email))
        .await
        .map(|user| {
            match verify_password(&data_pwd, &user.password) {
                Ok(true) => {
                    match generate_token(user) {
                        Ok(token) => HttpResponse::Created().json(TokenResponse {token}),
                        Err(e) => HttpResponse::InternalServerError().json(e.to_string()) 
                    }
                }
                Ok(false) => HttpResponse::Forbidden().json("Invalid credentials"),
                Err(e) => HttpResponse::InternalServerError().json(e.to_string())
            }
        })
        .map_err(|e| HttpResponse::BadRequest().json(e.to_string()))?)
}

// Handler for GET /users
pub async fn get_users(db: web::Data<Pool>) -> Result<HttpResponse, Error> {
    Ok(web::block(move || get_all_users(db))
        .await
        .map(|user| HttpResponse::Ok().json(user))
        .map_err(|_| HttpResponse::InternalServerError())?)
}

fn get_all_users(pool: web::Data<Pool>) -> Result<Vec<UserData>, diesel::result::Error> {
    let conn = pool.get().unwrap();
    let items = users.load::<User>(&conn)?.into_iter().map(|user| UserData {
        id: user.id,
        email: user.email,
        first_name: user.first_name,
        last_name: user.last_name,
        created_at: user.created_at
    }).collect();
    Ok(items)
}

// Handler for GET /users/{id}
pub async fn get_user_by_id(
    db: web::Data<Pool>,
    user_id: web::Path<String>,
    user_data: UserData
) -> Result<HttpResponse, Error> {
    let id_to_find = match user_id.parse::<i32>() {
        Ok(user_id) => user_id,
        Err(_e) => {
            if user_id.as_str() == "me" {
                user_data.id
            } else {
                return Ok(HttpResponse::BadRequest().json("Invalid user id"))
            }
        }
        
    };
    Ok(
        web::block(move || db_get_user_by_id(db, id_to_find))
            .await
            .map(|user| HttpResponse::Ok().json(user))
            .map_err(|_| ServiceError::NotFound)?,
    )
}

// Handler for POST /signup
pub async fn create_user(
    db: web::Data<Pool>,
    item: web::Json<InputUser>,
) -> Result<HttpResponse, Error> {
    Ok(web::block(move || add_single_user(db, item))
        .await
        .map(|user| match generate_token(user) {
            Ok(token) => HttpResponse::Created().json(TokenResponse {token}),
            Err(e) => HttpResponse::InternalServerError().json(e.to_string()) 
        })
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?)
}

// Handler for DELETE /users/{id}
pub async fn delete_user(
    db: web::Data<Pool>,
    user_id: web::Path<String>,
    user_data: UserData
) -> Result<HttpResponse, Error> {
    if user_id.as_str() != "me" {
        return Ok(HttpResponse::Forbidden().json("Can't delete another user"))
    }
    Ok(
        web::block(move || delete_single_user(db, user_data.id))
            .await
            .map(|_| HttpResponse::Ok().json("success"))
            .map_err(|e| HttpResponse::InternalServerError().json(e.to_string()))?,
    )
}

fn db_get_user_by_id(pool: web::Data<Pool>, user_id: i32) -> Result<UserData, diesel::result::Error> {
    let conn = pool.get().unwrap();
    users.find(user_id).get_result::<User>(&conn).map(|user| UserData {
        id: user.id,
        email: user.email,
        first_name: user.first_name,
        last_name: user.last_name,
        created_at: user.created_at
    })
}

fn db_get_user_by_email(pool: web::Data<Pool>, user_email: &str) -> Result<User, diesel::result::Error> {
    let conn = pool.get().unwrap();
    users.filter(email.eq(user_email)).first(&conn)
}

fn add_single_user(
    db: web::Data<Pool>,
    item: web::Json<InputUser>,
) -> Result<User, ServiceError> {
    let conn = db.get().unwrap();
    let encoded_pwd = &encode_password(&item.password).map_err(|e| ServiceError::InternalServerError(e.to_string()))?;
    let new_user = NewUser {
        first_name: &item.first_name,
        last_name: &item.last_name,
        email: &item.email,
        pwd: encoded_pwd,
        created_at: chrono::Local::now().naive_local(),
    };
    let res = insert_into(users).values(&new_user).get_result(&conn).map_err(|e| ServiceError::InternalServerError(e.to_string()))?;
    Ok(res)
}

fn delete_single_user(db: web::Data<Pool>, user_id: i32) -> Result<usize, diesel::result::Error> {
    let conn = db.get().unwrap();
    let count = delete(users.find(user_id)).execute(&conn)?;
    Ok(count)
}
