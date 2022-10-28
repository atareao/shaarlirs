use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use actix_web::web;
use sqlx::{sqlite::{SqlitePool, SqliteRow}, Error, query, Row};

use super::{metatag::Metatag, short_url};



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
    title: String,
    description: String,
    tags: Vec<String>,
    private: bool,
    created: DateTime<Utc>,
    updated: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LinkWithTagsNew {
    url: String,
    title: Option<String>,
    description: Option<String>,
    tags: Option<Vec<String>>,
    private: Option<bool>,
    created: Option<DateTime<Utc>>,
    updated: Option<DateTime<Utc>>,
}

impl Link{
    pub async fn create_from_post(pool: &web::Data<SqlitePool>, link_with_tags: LinkWithTagsNew) -> Result<LinkWithTags, Error>{
        let url = link_with_tags.url;
        let metatag = Metatag::new(&url).await.unwrap();
        let title = match link_with_tags.title {
            Some(title) => title,
            None => metatag.title,
        };
        let description = match link_with_tags.description {
            Some(description) => description,
            None => metatag.description,
        };
        let tags = match link_with_tags.tags {
            Some(tags) => tags,
            None => metatag.tags,
        };
        let private = match link_with_tags.private{
            Some(private) => private,
            None => true,
        };
        let created = match link_with_tags.created {
            Some(created) => created,
            None => Utc::now(),
        };
        let updated = match link_with_tags.created {
            Some(updated) => updated,
            None => Utc::now(),
        };
        let sql = "INSERT INTO links (url, title, description, private, created, updated) VALUES ($1, $2, $3, $4, $5, $6);";
        let id = query(sql)
            .bind(url)
            .bind(title)
            .bind(description)
            .bind(private)
            .bind(created)
            .bind(updated)
            .execute(pool.get_ref())
            .await?
        .last_insert_rowid();
        Self::set_shorturl(pool, id);
        let link = Self::read(id).await.unwrap();
        

    }
    async fn set_shorturl(pool: &web::Data<SqlitePool>, id: i64){
        let shorturl = short_url::encode(id.try_into().unwrap());
        let sql = "UPDATE links SET shorturl = $1 WHERE id = $2;";
        query(sql)
            .bind(shorturl)
            .bind(id)
            .execute(pool.get_ref())
            .await;
    }
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
