use actix_web::{get, post, put, delete, web, error::{ErrorNotFound,
    ErrorConflict, ErrorUnauthorized}, Error, HttpResponse};
use serde_json::json;
use sqlx::{SqlitePool, error::Error::Database};
use serde::Deserialize;
use log::debug;
use crate::models::history::History;

#[derive(Debug, Deserialize)]
struct Params{
    pub offset: Option<i32>,
    pub limit: Option<String>,
    pub since: Option<String>,
}


#[get("/history")]
pub async fn search(pool: web::Data<SqlitePool>, params: web::Query<Params>
) -> HttpResponse{
    debug!("Action: Search. Path: /tags");
    let offset = &params.offset;
    let limit = &params.limit;
    let since = &params.since;
    match History::search(&pool, since, offset, limit)
        .await{
            Ok(items) => HttpResponse::Ok().json(items),
            Err(_) => HttpResponse::BadRequest().json(
                json!({"code": 400, "message": "Invalid parameters"})),
        }
}
