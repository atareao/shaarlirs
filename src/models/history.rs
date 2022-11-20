use core::fmt;

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use actix_web::web;
use sqlx::{sqlite::{SqlitePool, SqliteRow}, Error, query, Row};
use log::debug;


enum Event{
    CREATED,
    UPDATED,
    DELETED,
    SETTINGS,
}

impl fmt::Display for Event{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        write!(f, "{}", self)
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct History {
    pub id: i64,
    pub event: String,
    pub datetime: DateTime<Utc>,
}

impl History{
    fn from_row(row: SqliteRow) -> History{
        History {
            id: row.get("id"),
            event: row.get("event"),
            datetime: row.get("datetime"),
        }
    }

    pub async fn created(pool: &web::Data<SqlitePool>) -> Result<History, Error>{
        Self::insert(pool, Event::CREATED).await
    }

    pub async fn updated(pool: &web::Data<SqlitePool>) -> Result<History, Error>{
        Self::insert(pool, Event::UPDATED).await
    }

    pub async fn deleted(pool: &web::Data<SqlitePool>) -> Result<History, Error>{
        Self::insert(pool, Event::DELETED).await
    }

    pub async fn settings(pool: &web::Data<SqlitePool>) -> Result<History, Error>{
        Self::insert(pool, Event::SETTINGS).await
    }

    async fn insert(pool: &web::Data<SqlitePool>, event: Event) -> Result<History, Error>{
        let datetime = Utc::now();
        let sql = "INSERT INTO history (event, datetime) VALUES ($1, $2) RETURNING *;";
        query(sql)
            .bind(event.to_string())
            .bind(datetime)
            .map(Self::from_row)
            .fetch_one(pool.get_ref())
            .await
    }

    pub async fn search(pool: &web::Data<SqlitePool>, since: &Option<String>, option_offset: &Option<i32>, option_limit: &Option<String>) -> Result<History, Error>{
        let offset = option_offset.unwrap_or(0);
        let limit = match option_limit {
            Some(v) => v.to_owned(),
            None => "20".to_string(),
        };
        let mut sql = Vec::new();
        sql.push("SELECT * FROM history".to_string());
        sql.push(match since{
            Some(v) => format!("WHERE datetime > '{}'", v),
            None => "".to_string(),
        });
        sql.push("ORDER BY datetime".to_string());
        sql.push(if limit != "all"{
            format!("LIMIT {} OFFSET {}", limit, offset)
        }else{
            "".to_string()
        });
        debug!("{}", &sql.join(" "));
        query(&sql.join(" "))
            .map(Self::from_row)
            .fetch_one(pool.get_ref())
            .await
    }
}
