use actix_web::{get, post, put, delete, web, error::{ErrorNotFound,
    ErrorConflict, ErrorUnauthorized}, Error, HttpResponse};
use sqlx::SqlitePool;
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
pub async fn create(pool: web::Data<SqlitePool>, new_link_with_tags: web::Json<LinkWithTagsNew>) -> Result<HttpResponse, Error>{
    Link::create_from_post(&pool, &new_link_with_tags)
        .await
        .map(|item| HttpResponse::Ok().json(item))
        .map_err(|e| ErrorConflict(e))
}

#[get("/links")]
pub async fn read(pool: web::Data<SqlitePool>, params: web::Query<Params>
) -> Result<HttpResponse, Error>{
    debug!("Path: /links");
    let offset = &params.offset;
    let limit = &params.limit;
    let searchterm = &params.searchterm;
    let searchtags = &params.searchtags;
    let visibility = &params.visibility;
    Link::search(&pool, offset, limit, searchterm, searchtags, visibility)
        .await
        .map(|item| HttpResponse::Ok().json(item))
        .map_err(|e| ErrorConflict(e))
}

#[get("/links/{link_id}")]
pub async fn read_one(pool: web::Data<SqlitePool>, link_id: web::Path<i64>,
) -> Result<HttpResponse, Error>{
    debug!("Path: /links/{}", link_id);
    Link::read(&pool, link_id.into_inner())
        .await
        .map(|item| HttpResponse::Ok().json(item))
        .map_err(|e| ErrorConflict(e))
}

#[put("/links/{link_id}")]
pub async fn update(pool: web::Data<SqlitePool>, link_id: web::Path<i64>, link_with_tags: web::Json<LinkWithTagsNew>
) -> Result<HttpResponse, Error>{
    Link::update(&pool, link_id.into_inner(), &link_with_tags)
        .await
        .map(|item| HttpResponse::Ok().json(item))
        .map_err(|e| ErrorConflict(e))
}

#[delete("/links/{link_id}")]
pub async fn delete(pool: web::Data<SqlitePool>, link_id: web::Path<i64>,
) -> Result<HttpResponse, Error>{
    Link::delete(&pool, link_id.into_inner())
        .await
        .map(|item| HttpResponse::Ok().json(item))
        .map_err(|e| ErrorConflict(e))
}

