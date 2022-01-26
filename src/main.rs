
#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate diesel;

extern crate env_logger;

extern crate bcrypt;

use actix_web::{dev::ServiceRequest, web, App, Error, error::InternalError, HttpMessage, HttpServer, middleware::Logger, HttpResponse};
use actix_web_httpauth::extractors::bearer::{BearerAuth, Config};
use actix_web_httpauth::extractors::AuthenticationError;
use actix_web_httpauth::middleware::HttpAuthentication;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

mod errors;
mod handlers;
mod models;
mod schema;
mod auth;

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

async fn validator(req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest, Error> {
    match auth::validate_token(credentials.token()) {
        Ok(claims) => {
            req.extensions_mut().insert(claims);
            Ok(req)
        },
        _ => {
            let config = req
                .app_data::<Config>()
                .map(|data| data.get_ref().clone())
                .unwrap_or_else(Default::default);
            Err(AuthenticationError::from(config).into())
        }
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().expect("Could not read .env file");
    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // create db connection pool
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool: Pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    // Start http server
    HttpServer::new(move || {
            let auth = HttpAuthentication::bearer(validator);
            App::new()
                .wrap(Logger::default())
                .data(pool.clone())
                .app_data(web::JsonConfig::default().error_handler(|err, _req| {
                    InternalError::from_response(
                        "",
                        HttpResponse::BadRequest()
                            .content_type("application/json")
                            .body(format!(r#"{{"error":"{}"}}"#, err)),
                    ).into()
                }))
                .route("/signup", web::post().to(handlers::create_user))
                .route("/login", web::post().to(handlers::login))
                .service(
                    // routes needing bearer auth
                    web::scope("/")
                        .wrap(auth)
                        .route("/users", web::get().to(handlers::get_users))
                        .route("/users/{id}", web::get().to(handlers::get_user_by_id))
                        .route("/users/{id}", web::delete().to(handlers::delete_user))
                )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}