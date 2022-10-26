use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use actix_web::web;
use sqlx::{sqlite::{SqlitePool, SqliteRow}, Error, query, Row};



#[derive(Debug, Serialize, Deserialize)]
pub struct Link {
    id: i64,
    url: String,
    shorturl: String,
    title: String,
    description: String,
    private: bool,
    created: DateTime<Utc>,
    updated: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LinkWithTags {
    id: i64,
    url: String,
    shorturl: String,
    title: String,
    description: String,
    tags: Vec<String>,
    private: bool,
    created: DateTime<Utc>,
    updated: DateTime<Utc>,
}

impl Link{
    pub async fn create(pool: &web::Data<SqlitePool>, url: &str) -> Result<Link, Error>{
        
        let sql = "INSERT INTO links (url, shorturl, title, description, private, created, updated) VALUES ($1, $2, $3, $4, $5, $6, $7);";
        let id = query(sql)
            .execute(pool.get_ref())
            .await?
        .last_insert_rowid();
        Self::read(pool, id).await
    }

    pub async fn read(pool: &web::Data<SqlitePool>, id: i64) -> Result<Link, Error>{
        let sql = "SELECT id, url, shorturl, title, description, private, created, updated FROM links WHERE id = $1;";
        query(sql)
            .bind(id)
            .map(|row: SqliteRow| Link{
                id: row.get("id"),
                url: row.get("url"),
                shorturl: row.get("shorturl"),
                title: row.get("title"),
                description: row.get("description"),
                private: row.get("private"),
                created: row.get("created"),
                updated: row.get("updated"),
            })
            .fetch_one(pool.get_ref())
            .await
    }
}
