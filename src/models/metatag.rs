
use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct Metatag {
    url: String,
    title: String,
    description: String,
    tags: Vec<String>,
}

impl Metatag {
    pub async fn new(url: &str) -> Self{

    }
}
