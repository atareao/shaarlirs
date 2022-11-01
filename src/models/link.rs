use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use actix_web::web;
use sqlx::{sqlite::{SqlitePool, SqliteRow, SqliteQueryResult}, Error, query, Row};

use super::{metatag::Metatag, short_url, tag::Tag, link_tag::LinkTag};



#[derive(Debug, Serialize, Deserialize, Eq)]
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

impl PartialEq for Link{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
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

    pub async fn create_from_post(pool: &web::Data<SqlitePool>, link_with_tags: &LinkWithTagsNew) -> Result<LinkWithTags, Error>{
        let url = &link_with_tags.url;
        let metatag = Metatag::new(&url).await.unwrap();
        let title = match &link_with_tags.title {
            Some(title) => title,
            None => &metatag.title,
        };
        let description = match &link_with_tags.description {
            Some(description) => description,
            None => &metatag.description,
        };
        let tags_names = match &link_with_tags.tags {
            Some(tags) => tags,
            None => &metatag.tags,
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
        let sql = "INSERT INTO links (url, shorturl, title, description,
                   private, created, updated) VALUES ($1, $2, $3, $4, $5, $6,
                   $7) RETURNING * ;";
        let link = query(sql)
            .bind(url)
            .bind("")
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
            let _ = LinkTag::create(pool, link.id, tag.id).await;
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
    pub async fn create(pool: &web::Data<SqlitePool>, url: &str) -> Result<LinkWithTags, Error>{
        let metatag = Metatag::new(&url).await.unwrap();
        println!("{}", &metatag);
        let title = metatag.title;
        let description = metatag.description;
        let tags_names = metatag.tags;
        let private = true;
        let created = Utc::now();
        let updated = created;
        let link_with_tags = LinkWithTagsNew {
            url: url.to_string(),
            title: Some(title),
            description: Some(description),
            tags: Some(tags_names),
            private: Some(private),
            created: Some(created),
            updated: Some(updated),
        };
        Self::create_from_post(pool, &link_with_tags).await
    }

    pub async fn read(pool: &web::Data<SqlitePool>, id: i64) -> Result<Link, Error>{
        let sql = "SELECT * FROM links WHERE id = $1;";
        query(sql)
            .bind(id)
            .map(Self::from_row)
            .fetch_one(pool.get_ref())
            .await
    }

    pub async fn read_all(pool: &web::Data<SqlitePool>) -> Result<Vec<Link>, Error>{
        let sql = "SELECT * FROM links;";
        query(sql)
            .map(Self::from_row)
            .fetch_all(pool.get_ref())
            .await
    }

    pub async fn search(pool: &web::Data<SqlitePool>, 
            option_offset: web::Path<Option<i32>>,
            option_limit: web::Path<Option<&str>>,
            option_searchterm: web::Path<Option<&str>>,
            option_searchtags: web::Path<Option<&str>>,
            option_visibility: web::Path<Option<&str>>,
        ) -> Result<Vec<Link>, Error>{
        let offset = option_offset.unwrap_or(0);
        let limit = option_limit.unwrap_or("0");
        let mut sql = Vec::new();
        let mut conditions = Vec::new();
        sql.push("SELECT l.* FROM links ORDER BY id".to_string());
        conditions.push(match option_searchterm.into_inner(){
            Some(value) => format!("l.title LIKE '%{}%' or l.description LIKE '%{}%'", value, value),
            None => "1 = 1".to_string(),
        });
        let (sql2, condition2) = match option_searchtags.into_inner(){
            Some(value) => {
                let tags = value.split("+").map(|x| format!("'{}'", x.trim())).collect::<Vec<String>>().join(",");
                ("INNER JOIN links_tags lt ON l.id = lt.link_i
                  INNER JOIN tags t ON t.id = lt.tag_id".to_string(),
                format!("t.name IN ({})", tags))
            },
            None => ("".to_string(), "1 = 1".to_string()),
        };
        sql.push(sql2);
        conditions.push(condition2);
        conditions.push(match option_visibility.into_inner(){
            Some(value) => {
                if value == "all" {
                    "1 = 1".to_string()
                }else{
                    format!("private = {}", "private" == value)
                }
            },
            None => "1 = 1".to_string(),
        });
        sql.push(format!("WHERE {}", conditions.join(" AND ")));
        sql.push(if limit != "all"{
            format!("OFFSET {} FETCH NEXT {} ROWS ONLY", offset, limit)
        }else{
            "".to_string()
        });
        println!("{}", &sql.join(" "));
        query(&sql.join(" "))
            .map(Self::from_row)
            .fetch_all(pool.get_ref())
            .await
    }

    pub async fn update(pool: &web::Data<SqlitePool>, link_id: i64, link_with_tags: &LinkWithTagsNew) -> Result<Link, Error>{
        let sql = "UPDATE links SET url = $1, title = $2, description = $3,
                   private = $4, created = $5, updated = &6 WHERE id = $7
                   RETURNING *";
        query(sql)
            .bind(&link_with_tags.url)
            .bind(&link_with_tags.title)
            .bind(&link_with_tags.description)
            .bind(&link_with_tags.private)
            .bind(&link_with_tags.created)
            .bind(&link_with_tags.updated)
            .bind(link_id)
            .map(Self::from_row)
            .fetch_one(pool.get_ref())
            .await
    }

    pub async fn delete(pool: &web::Data<SqlitePool>, link_id: i64) -> Result<Link, Error>{
        let sql = "DELETE FROM links WHERE id = $1 RETURNING *;";
        query(sql)
            .bind(link_id)
            .map(Self::from_row)
            .fetch_one(pool.get_ref())
            .await
    }

    pub async fn drop(pool: &web::Data<SqlitePool>) -> Result<SqliteQueryResult, Error>{
        let sql = "DELETE FROM links;";
        query(sql)
            .execute(pool.get_ref())
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::{env, path::Path};
    use sqlx::{Sqlite, Pool, sqlite::SqlitePoolOptions, migrate::{Migrator,
        MigrateDatabase}};
    use actix_web::web::Data;
    use super::{Tag, Link, LinkWithTagsNew};
    use dotenv::dotenv;

    async fn setup() -> Data<Pool<Sqlite>>{
        dotenv().ok();
        let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
        if !Sqlite::database_exists(&db_url).await.unwrap(){
            Sqlite::create_database(&db_url).await.unwrap()
        }
        let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let migrations = Path::new(&crate_dir).join("migrations");
        println!("{}", migrations.to_str().unwrap());

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&db_url)
        .await
        .expect("pool failed");

    Migrator::new(migrations)
        .await.unwrap()
        .run(&pool)
        .await.unwrap();

        Data::new(pool)
    }

    async fn teardown(pool: &Data<Pool<Sqlite>>){
        let _result = Link::drop(pool).await;
    }

    #[tokio::test]
    async fn create(){
        let pool = setup().await;
        let new_link = LinkWithTagsNew {
            url: "https://atareao.es".to_string(),
            title: None,
            description: None,
            tags: None,
            private: None,
            created: None,
            updated: None,
        };

        let url = "https://atareao.es";
        match Link::create(&pool, url).await {
            Ok(link) => {
                assert_eq!(&link.url, &new_link.url);
            },
            Err(_) => assert!(false),
        }
        teardown(&pool).await;
    }

    #[tokio::test]
    async fn read_one(){
        let pool = setup().await;
        match Link::create(&pool, "https://atareao.es").await {
            Ok(link) => {
                let test = Link::read(&pool, link.id).await.unwrap();
                assert_eq!(test.id, link.id);
            },
            Err(_) => assert!(false),
        }
        teardown(&pool).await;
    }

    #[tokio::test]
    async fn read_all(){
        let pool = setup().await;
        let _ = Link::create(&pool, "https://atareao.es").await;
        let _ = Link::create(&pool, "https://google.es").await;
        let _ = Link::create(&pool, "https://github.com").await;
        let links = Link::read_all(&pool).await.unwrap();
        assert_eq!(links.len(), 3);
        teardown(&pool).await;
    }

    #[tokio::test]
    async fn delete(){
        let pool = setup().await;
        let link = Link::create(&pool, "etiqueta 4").await.unwrap();
        let _ = Link::delete(&pool, link.id).await;
        let tags = Link::read_all(&pool).await.unwrap();
        assert_eq!(tags.len(), 0);
        teardown(&pool).await;
    }
}
