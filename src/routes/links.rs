use actix_web::{get, post, put, delete, web, error::{ErrorNotFound,
    ErrorConflict, ErrorUnauthorized}, Error, HttpResponse};
use serde_json::json;
use sqlx::{SqlitePool, error::Error::Database};
use serde::Deserialize;
use log::debug;

use crate::models::link::{LinkWithTagsNew, Link};
#[derive(Debug, Deserialize)]
struct Params{
    pub offset: Option<i32>,
    pub limit: Option<String>,
    pub searchterm: Option<String>,
    pub searchtags: Option<String>,
    pub visibility: Option<String>,
}


#[post("/links")]
pub async fn create(pool: web::Data<SqlitePool>, new_link_with_tags: web::Json<LinkWithTagsNew>) -> HttpResponse{
    match Link::create_from_post(&pool, &new_link_with_tags).await{
        Ok(item) => HttpResponse::Created().json(item),
        Err(e) => {
            if let Some(err) = e.as_database_error(){
                debug!("{:?}", err);
                HttpResponse::Conflict().json(
                    json!({"code": 409, "message": e.to_string()}))
            }else{
                HttpResponse::BadRequest().json(
                    json!({"code": 400, "message": e.to_string()}))
            }
        },
    }
}

#[get("/links")]
pub async fn read(pool: web::Data<SqlitePool>, params: web::Query<Params>
) -> HttpResponse{
    debug!("Path: /links");
    let offset = &params.offset;
    let limit = &params.limit;
    let searchterm = &params.searchterm;
    let searchtags = &params.searchtags;
    let visibility = &params.visibility;
    match Link::search(&pool, offset, limit, searchterm, searchtags, visibility)
        .await{
            Ok(items) => HttpResponse::Ok().json(items),
            Err(_) => HttpResponse::BadRequest().json(
                json!({"code": 400, "message": "Invalid parameters"})),
        }
}

#[get("/links/{link_id}")]
pub async fn read_one(pool: web::Data<SqlitePool>, link_id: web::Path<i64>,
) -> HttpResponse{
    debug!("Path: /links/{}", link_id);
    match Link::read(&pool, link_id.into_inner()).await{
        Ok(item) => HttpResponse::Ok().json(item),
        Err(_) => HttpResponse::BadRequest().finish(),
    }
}

#[put("/links/{link_id}")]
pub async fn update(pool: web::Data<SqlitePool>, link_id: web::Path<i64>, link_with_tags: web::Json<LinkWithTagsNew>
) -> HttpResponse {
    match Link::update(&pool, link_id.into_inner(), &link_with_tags).await{
        Ok(item) => HttpResponse::Ok().json(item),
        Err(_) => HttpResponse::BadRequest().finish(),
    }
}

#[delete("/links/{link_id}")]
pub async fn delete(pool: web::Data<SqlitePool>, link_id: web::Path<i64>,
) -> HttpResponse {
    match Link::delete(&pool, link_id.into_inner()).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(_) => HttpResponse::BadRequest().finish(),
    }
}

