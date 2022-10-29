use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use actix_web::web;
use sqlx::{sqlite::{SqlitePool, SqliteRow}, Error, query, Row};

use super::{metatag::Metatag, short_url, tag::Tag, link_tag::LinkTag};



#[derive(Debug, Serialize, Deserialize)]
pub struct Link {
    pub id: i64,
    pub url: String,
    pub shorturl: String,
    pub title: String,
    pub description: String,
    pub private: bool,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
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
    fn from_row(row: SqliteRow) -> Link{
        Link{
            id: row.get("id"),
            url: row.get("url"),
            shorturl: row.get("shorturl"),
            title: row.get("title"),
            description: row.get("description"),
            private: row.get("private"),
            created: row.get("created"),
            updated: row.get("updated"),
        }
    }

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
        let tags_names = match link_with_tags.tags {
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
        let sql = "INSERT INTO links (url, title, description, private,
                   created, updated) VALUES ($1, $2, $3, $4, $5, $6)
                   RETURNING * ;";
        let link = query(sql)
            .bind(url)
            .bind(title)
            .bind(description)
            .bind(private)
            .bind(created)
            .bind(updated)
            .map(Self::from_row)
            .fetch_one(pool.get_ref())
            .await?;
        let shorturl = Self::set_shorturl(pool, link.id).await;
        for tag_name in tags_names{
            let tag = Tag::get_or_insert(&pool, &tag_name).await.unwrap();
            let link_tag = LinkTag::create(pool, link.id, tag.id).await;
        }
        let tags = Tag::read_tags_for_link(&pool, link.id).await.unwrap();
        Ok(LinkWithTags {
            id: link.id,
            url: link.url,
            shorturl,
            title: link.title,
            description: link.description,
            tags,
            private: link.private,
            created: link.created,
            updated: link.updated,
        })
    }
    async fn set_shorturl(pool: &web::Data<SqlitePool>, id: i64) -> String{
        let shorturl = short_url::encode(id.try_into().unwrap());
        let sql = "UPDATE links SET shorturl = $1 WHERE id = $2;";
        query(sql)
            .bind(&shorturl)
            .bind(id)
            .execute(pool.get_ref())
            .await;
        shorturl
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
