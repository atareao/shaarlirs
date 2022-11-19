use actix_web::{get, web, HttpResponse, Error};
use sqlx::SqlitePool;
use log::debug;



#[get("/api/v1/info")]
pub async fn get_info(req: actix_web::HttpRequest, pool: web::Data<SqlitePool>) -> Result<HttpResponse, Error>{
    debug!("Reques: {:?}", req);
    Ok(HttpResponse::Ok().finish())
}
