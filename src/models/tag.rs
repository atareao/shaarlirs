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
    pub async fn create(pool: &web::Data<SqlitePool>, name: &str) -> Result<Tag, Error>{
        
        let sql = "INSERT INTO tags (name) VALUES ($1);";
        let id = query(sql)
            .bind(name)
            .execute(pool.get_ref())
            .await?
        .last_insert_rowid();
        Self::read(pool, id).await
    }

    pub async fn read(pool: &web::Data<SqlitePool>, id: i64) -> Result<Tag, Error>{
        let sql = "SELECT id, name FROM tags WHERE id = $1;";
        query(sql)
            .bind(id)
            .map(|row: SqliteRow| Tag{
                id: row.get("id"),
                name: row.get("name"),
            })
            .fetch_one(pool.get_ref())
            .await
    }
}

