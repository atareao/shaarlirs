use actix_web::{get, post, put, delete, web, error::{ErrorNotFound,
    ErrorConflict, ErrorUnauthorized}, Error, HttpResponse};
use sqlx::SqlitePool;

use crate::models::link::{LinkWithTagsNew, Link};


#[post("/links")]
pub async fn create(pool: web::Data<SqlitePool>, new_link_with_tags: web::Json<LinkWithTagsNew>) -> Result<HttpResponse, Error>{
    Link::create_from_post(&pool, &new_link_with_tags)
        .await
        .map(|item| HttpResponse::Ok().json(item))
        .map_err(|e| ErrorConflict(e))
}

#[get("/links")]
pub async fn read(pool: web::Data<SqlitePool>,
        offset: web::Path<Option<i32>>,
        limit: web::Path<Option<String>>,
        searchterm: web::Path<Option<String>>,
        searchtags: web::Path<Option<String>>,
        visibility: web::Path<Option<String>>,
) -> Result<HttpResponse, Error>{
    Link::search(&pool, offset, limit, searchterm, searchtags, visibility)
        .await
        .map(|item| HttpResponse::Ok().json(item))
        .map_err(|e| ErrorConflict(e))
}
