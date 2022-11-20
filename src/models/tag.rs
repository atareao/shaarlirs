use serde::{Serialize, Deserialize};
use actix_web::web;
use sqlx::{sqlite::{SqlitePool, SqliteRow, SqliteQueryResult}, Error, query, Row};
use log::debug;



#[derive(Debug, Serialize, Deserialize, Eq)]
pub struct Tag {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TagWithOccurrences {
    name: String,
    occurrences: i64,
}

impl PartialEq for Tag {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.name == other.name
    }
}

impl Tag{
    fn from_row(row: SqliteRow) -> Tag{
        Tag {
            id: row.get("id"),
            name: row.get("name"),
        }
    }
    fn from_row_with_ocurrences(row: SqliteRow) -> TagWithOccurrences {
        TagWithOccurrences {
            name: row.get("name"),
            occurrences: row.get("occurrences"),
        }
    }
    fn get_string(row: SqliteRow) -> String{
        row.get("name")
    }

    pub async fn get_or_insert(pool: &web::Data<SqlitePool>, name: &str) -> Result<Tag, Error>{
        match Self::read_from_name(pool, name).await {
            Ok(tag) => Ok(tag),
            Err(_) => Self::create(pool, name).await
        }
    }

    pub async fn create(pool: &web::Data<SqlitePool>, name: &str) -> Result<Tag, Error>{
        let sql = "INSERT INTO tags (name) VALUES ($1) RETURNING *;";
        query(sql)
            .bind(name)
            .map(Self::from_row)
            .fetch_one(pool.get_ref())
            .await
    }

    pub async fn search(pool: &web::Data<SqlitePool>, option_offset: &Option<i32>, option_limit: &Option<String>, _option_visibility: &Option<String>) -> Result<TagWithOccurrences, Error>{
        let offset = option_offset.unwrap_or(0);
        let limit = match option_limit {
            Some(v) => v.to_owned(),
            None => "20".to_string(),
        };
        let mut sql = Vec::new();
        sql.push("SELECT t.name name, count(*) occurrences FROM tags t
                   LEFT JOIN links_tags lt ON t.id = lt.tag_id
                   GROUP BY t.name
                   ORDER BY t.name".to_string());
        sql.push(if limit != "all"{
            format!("LIMIT {} OFFSET {}", limit, offset)
        }else{
            "".to_string()
        });
        debug!("{}", &sql.join(" "));
        query(&sql.join(" "))
            .map(Self::from_row_with_ocurrences)
            .fetch_one(pool.get_ref())
            .await
    }
    pub async fn read(pool: &web::Data<SqlitePool>, name: &str) -> Result<TagWithOccurrences, Error>{
        let sql = "SELECT t.name name, count(*) occurrences FROM tags t
                   LEFT JOIN links_tags lt ON t.id = lt.tag_id
                   WHERE t.name = $1
                   GROUP BY t.name";
        query(sql)
            .bind(name)
            .map(Self::from_row_with_ocurrences)
            .fetch_one(pool.get_ref())
            .await
    }

    pub async fn read_all(pool: &web::Data<SqlitePool>) -> Result<Vec<Tag>, Error>{
        let sql = "SELECT * FROM tags;";
        query(sql)
            .map(Self::from_row)
            .fetch_all(pool.get_ref())
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

    pub async fn read_tags_for_link(pool: &web::Data<SqlitePool>, link_id: i64) -> Result<Vec<String>, Error>{
        let sql = "SELECT name FROM tags t
                   INNER JOIN links_tags lt on t.id = lt.tag_id
                   WHERE  lt.link_id = $1;";
        query(sql)
            .bind(link_id)
            .map(|row: SqliteRow| row.get("name"))
            .fetch_all(pool.get_ref())
            .await
    }
    
    pub async fn delete(pool: &web::Data<SqlitePool>, name: &str) -> Result<Tag, Error>{
        let sql = "DELETE FROM tags WHERE name = $1 RETURNING *;";
        query(sql)
            .bind(name)
            .map(Self::from_row)
            .fetch_one(pool.get_ref())
            .await
    }

    pub async fn update(pool: &web::Data<SqlitePool>, name: &str, new_name: &str) -> Result<TagWithOccurrences, Error>{
        let sql = "UPDATE tags SET name = $1 WHERE name = $2 RETURNING *;";
        match query(sql)
            .bind(new_name)
            .bind(name)
            .map(Self::from_row)
            .fetch_one(pool.get_ref())
            .await{
                Ok(_) => Self::read(pool, new_name).await,
                Err(e) => Err(e),
        }
    }

    pub async fn drop(pool: &web::Data<SqlitePool>) -> Result<SqliteQueryResult, Error>{
        let sql = "DELETE FROM tags";
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
    use super::Tag;
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
        let _result = Tag::drop(pool).await;
    }

    #[tokio::test]
    async fn create(){
        let pool = setup().await;
        match Tag::create(&pool, "etiqueta").await {
            Ok(tag) => {
                assert_eq!(tag.name, "etiqueta");
            },
            Err(_) => assert!(false),
        }
        teardown(&pool).await;
    }

    #[tokio::test]
    async fn read_one(){
        let pool = setup().await;
        match Tag::create(&pool, "etiqueta").await {
            Ok(tag) => {
                let test = Tag::read(&pool, "etiqueta").await.unwrap();
                assert_eq!(test.name, tag.name);
            },
            Err(_) => assert!(false),
        }
        teardown(&pool).await;
    }

    #[tokio::test]
    async fn read_all(){
        let pool = setup().await;
        let _ = Tag::create(&pool, "etiqueta 1").await;
        let _ = Tag::create(&pool, "etiqueta 2").await;
        let _ = Tag::create(&pool, "etiqueta 3").await;
        let tags = Tag::read_all(&pool).await.unwrap();
        assert_eq!(tags.len(), 3);
        teardown(&pool).await;
    }

    #[tokio::test]
    async fn delete(){
        let pool = setup().await;
        let tag = Tag::create(&pool, "etiqueta 4").await.unwrap();
        let _ = Tag::delete(&pool, &tag.name).await;
        let tags = Tag::read_all(&pool).await.unwrap();
        assert_eq!(tags.len(), 0);
        teardown(&pool).await;
    }
}
