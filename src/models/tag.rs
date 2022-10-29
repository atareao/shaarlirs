use serde::{Serialize, Deserialize};
use actix_web::web;
use sqlx::{sqlite::{SqlitePool, SqliteRow}, Error, query, Row};



#[derive(Debug, Serialize, Deserialize)]
pub struct Tag {
    id: i64,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TagWithOccurrences {
    name: String,
    occurrences: i64,
}

impl Tag{
    fn from_row(row: SqliteRow) -> Tag{
        Tag {
            id: row.get("id"),
            name: row.get("name"),
        }
    }

    pub async fn get_or_insert(pool: &web::Data<SqlitePool>, name: &str) -> Result<Tag, Error>{
        match Self::read_from_name(pool, name).await {
            Ok(tag) => Ok(tag),
            Err(_) => Self::create(pool, name).await
        }
    }

    pub async fn create(pool: &web::Data<SqlitePool>, name: &str) -> Result<Tag, Error>{
        let sql = "INSERT INTO tags (name) VALUES ($1) RETURNING id, name;";
        query(sql)
            .bind(name)
            .map(Self::from_row)
            .fetch_one(pool.get_ref())
            .await
    }

    pub async fn read(pool: &web::Data<SqlitePool>, id: i64) -> Result<Tag, Error>{
        let sql = "SELECT id, name FROM tags WHERE id = $1;";
        query(sql)
            .bind(id)
            .map(Self::from_row)
            .fetch_one(pool.get_ref())
            .await
    }

    pub async fn read_from_name(pool: &web::Data<SqlitePool>, name: &str) -> Result<Tag, Error>{
        let sql = "SELECT id, name FROM tags WHERE name = $1;";
        query(sql)
            .bind(name)
            .map(Self::from_row)
            .fetch_one(pool.get_ref())
            .await
    }
}

