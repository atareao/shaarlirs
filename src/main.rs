use actix_web::{HttpServer, App, web::{self, Data}, middleware::Logger};
use sqlx::{query, sqlite::{SqlitePool, SqlitePoolOptions}, migrate::{Migrator, MigrateDatabase}};
use std::{env, path::Path, process};
use tokio::fs;
use env_logger::Env;
use log::{debug, error};
use tera::Tera;
use actix_web_httpauth::extractors::basic;
use actix_files;
use dotenv::dotenv;

mod models;
mod routes;



#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let log_level = env::var("LOG_LEVEL").expect("LOG_LEVEL not set");
    env_logger::init_from_env(Env::default().default_filter_or(&log_level));
    debug!("Log level: {}", &log_level);
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
    debug!("DB url: {}", db_url);
    let port = env::var("PORT").expect("PORT not set");
    debug!("Port: {}", port);

    let template = match Tera::new("templates/**/*.html"){
        Ok(t) => t,
        Err(e) => {
            error!("Can not load templates, {}", e);
            process::exit(1);
        }
    };
    if !sqlx::Sqlite::database_exists(&db_url).await.unwrap(){
        sqlx::Sqlite::create_database(&db_url).await.unwrap();
    }

    let migrations = if env::var("RUST_ENV") == Ok("production".to_string()){
        std::env::current_exe().unwrap().parent().unwrap().join("migrations")
    }else{
        let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        Path::new(&crate_dir).join("./migrations")
    };
    debug!("{}", &migrations.display());

    let pool = SqlitePoolOptions::new()
        .max_connections(4)
        .connect(&db_url)
        .await
        .expect("Pool failed");

    Migrator::new(migrations)
        .await
        .unwrap()
        .run(&pool)
        .await
        .unwrap();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
.app_data(Data::new(pool.clone()))
            .app_data(Data::new(template.clone()))
            //.service(routes::get_form)
            .service(routes::general::get_info)
            .service(
                web::scope("results")
                .app_data(basic::Config::default().realm("Restricted area"))
                )
                //.service(routes::get_results))
            .service(actix_files::Files::new("/static", "./static"))
    })
    .workers(4)
    .bind(format!("0.0.0.0:{}", &port))
    .unwrap()
    .run()
    .await
}
