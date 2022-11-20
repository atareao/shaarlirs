use actix_web::{get, post, put, delete, web, error::{ErrorNotFound,
    ErrorConflict, ErrorUnauthorized}, Error, HttpResponse};
use serde_json::json;
use sqlx::{SqlitePool, error::Error::Database};
use serde::Deserialize;
use log::debug;
use crate::models::tag::Tag;

#[derive(Debug, Deserialize)]
struct Params{
    pub offset: Option<i32>,
    pub limit: Option<String>,
    pub visibility: Option<String>,
}

#[get("/tags")]
pub async fn search(pool: web::Data<SqlitePool>, params: web::Query<Params>
) -> HttpResponse{
    debug!("Path: /tags");
    let offset = &params.offset;
    let limit = &params.limit;
    let visibility = &params.visibility;
    match Tag::search(&pool, offset, limit, visibility)
        .await{
            Ok(items) => HttpResponse::Ok().json(items),
            Err(_) => HttpResponse::BadRequest().json(
                json!({"code": 400, "message": "Invalid parameters"})),
        }
}

#[get("/tags/{name}")]
pub async fn read(pool: web::Data<SqlitePool>, name: web::Path<String>) -> HttpResponse{
    debug!("Path: /tags{name}");
    match Tag::read(&pool, &name)
        .await{
            Ok(items) => HttpResponse::Ok().json(items),
            Err(_) => HttpResponse::BadRequest().json(
                json!({"code": 400, "message": "Invalid parameters"})),
        }
}

