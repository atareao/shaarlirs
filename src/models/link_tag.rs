use serde::{Serialize, Deserialize};
use sqlx::{sqlite::{SqlitePool, SqliteRow, SqliteQueryResult}, Error, query, Row};

#[derive(Debug, Serialize, Deserialize, Eq)]
pub struct LinkTag{
    pub id: i64,
    pub link_id: i64,
    pub tag_id: i64,
}

impl PartialEq for LinkTag{
    fn eq(&self, other: &Self) -> bool{
        self.id == other.id && self.link_id == other.link_id &&
            self.tag_id == other.tag_id
    }
}

impl LinkTag {
    fn from_row(row: SqliteRow) -> LinkTag{
        Self{
            id: row.get("id"),
            link_id: row.get("link_id"),
            tag_id: row.get("tag_id"),
        }
    }

    pub async fn create(pool: &SqlitePool, link_id: i64, tag_id: i64) -> Result<LinkTag, Error>{
        let sql = "INSERT INTO links_tags (link_id, tag_id) VALUES ($1, $2)
                   RETURNING *";
        query(sql)
            .bind(link_id)
            .bind(tag_id)
            .map(Self::from_row)
            .fetch_one(pool)
            .await
    }

    pub async fn read(pool: &SqlitePool, id: i64) -> Result<LinkTag, Error>{
        let sql = "SELECT * FROM links_tags WHERE id = $1";
        query(sql)
            .bind(id)
            .map(Self::from_row)
            .fetch_one(pool)
            .await
    }

    pub async fn read_all(pool: &SqlitePool) -> Result<Vec<LinkTag>, Error>{
        let sql = "SELECT * FROM links_tags";
        query(sql)
            .map(Self::from_row)
            .fetch_all(pool)
            .await
    }

    pub async fn update(pool: &SqlitePool, id: i64, link_id: i64, tag_id: i64) -> Result<LinkTag, Error>{
        let sql = "UPDATE links_tags SET link_id = $1, tag_id =$2 WHERE id = $3 RETURNING *;";
        query(sql)
            .bind(link_id)
            .bind(tag_id)
            .bind(id)
            .map(Self::from_row)
            .fetch_one(pool)
            .await
    }

    pub async fn delete(pool: &SqlitePool, id: i64) -> Result<LinkTag, Error>{
        let sql = "DELETE FROM links_tags WHERE id = $1 RETURNING *;";
        query(sql)
            .bind(id)
            .map(Self::from_row)
            .fetch_one(pool)
            .await
    }

    pub async fn drop(pool: &SqlitePool) -> Result<SqliteQueryResult, Error>{
        let sql = "DELETE FROM links_tags;";
        query(sql)
            .execute(pool)
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::{env, path::Path};
    use sqlx::{Sqlite, Pool, sqlite::SqlitePoolOptions, migrate::{Migrator,
        MigrateDatabase}};
    use super::LinkTag;
    use dotenv::dotenv;

    async fn setup() -> Pool<Sqlite>{
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

        pool
    }

    async fn teardown(pool: &Pool<Sqlite>){
        let _result = LinkTag::drop(pool).await;
    }

    #[tokio::test]
    async fn create(){
        let pool = setup().await;
        match LinkTag::create(&pool, 1, 1).await {
            Ok(linktag) => {
                assert_eq!(linktag.link_id, 1);
                assert_eq!(linktag.tag_id, 1);
            },
            Err(_) => assert!(false),
        }
        teardown(&pool).await;
    }

    #[tokio::test]
    async fn read_one(){
        let pool = setup().await;
        match LinkTag::create(&pool, 1, 1).await {
            Ok(linktag) => {
                let test = LinkTag::read(&pool, linktag.id).await.unwrap();
                assert_eq!(test, linktag);
            },
            Err(_) => assert!(false),
        }
        teardown(&pool).await;
    }

    #[tokio::test]
    async fn read_all(){
        let pool = setup().await;
        let _ = LinkTag::create(&pool, 1, 1).await;
        let _ = LinkTag::create(&pool, 1, 2).await;
        let _ = LinkTag::create(&pool, 1, 3).await;
        let links_tags = LinkTag::read_all(&pool).await.unwrap();
        assert_eq!(links_tags.len(), 3);
        teardown(&pool).await;
    }

    #[tokio::test]
    async fn delete(){
        let pool = setup().await;
        let link_tag = LinkTag::create(&pool, 1, 1).await.unwrap();
        let _ = LinkTag::delete(&pool, link_tag.id).await;
        let links_tags = LinkTag::read_all(&pool).await.unwrap();
        assert_eq!(links_tags.len(), 0);
        teardown(&pool).await;
    }
}
