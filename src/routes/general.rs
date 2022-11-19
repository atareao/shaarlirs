use actix_web::{get, web, HttpResponse, Error, error::ErrorForbidden};
use sqlx::SqlitePool;
use serde::{Serialize, Deserialize};
use crate::models::{claim::authorize, general::{Info, Settings}};
use serde_json;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    iat: i64,
}



#[get("/api/v1/info")]
pub async fn get_info(req: actix_web::HttpRequest, secret: web::Data<String>, pool: web::Data<SqlitePool>) -> Result<HttpResponse, Error>{
    match authorize(&req.headers(), &secret){
        Ok(_) => {
            let settings = Settings::new("Título", "enlace", "Europe/Madrid", Vec::new(), true);
            let info = Info::new(0, 0, settings);
            Ok(HttpResponse::Ok()
                .body(serde_json::to_string(&info).unwrap())
            )
        },
        Err(e) => Err(ErrorForbidden(e)),
    }
}
