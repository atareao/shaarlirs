use regex::Regex;
use reqwest::{Error, Client, header::USER_AGENT};
use serde::{Serialize, Deserialize};
use std::fmt;


#[derive(Debug, Serialize, Deserialize)]
pub struct Metatag {
    url: String,
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
}

impl Metatag {
    pub async fn new(url: &str) -> Result<Self, Error>{
        let client = Client::new();
        let content = client.get(url)
            .header(USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 12_1) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.0.4664.110 Safari/537.36")
            .send()
            .await?
            .text()
            .await?;
        let title = search_for_title("<title>", "</title>", &content);
        let description = match search_for_meta("description", &content){
            Some(value) => value[1].to_string(),
            None => "".to_string(),
        };
        let keywords = match search_for_meta("keywords", &content){
            Some(tags) => tags[1].split(",").map(|item| item.trim().to_string()).collect(),
            None => Vec::new(),
        };
        Ok(Self{
            url: url.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            tags: keywords,
        })
    }
}

impl fmt::Display for Metatag{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "url: {}\ntitle: {}\ndescription: {}\ntags: {}",
            self.url, self.title, self.description, self.tags.join(", "))
    }
}


fn search_for_title<'a>(open: &'a str, close: &'a str, s: &'a str) -> &'a str
{
    let part_1 = s.splitn(3, &open[..open.len()-1])
                  .nth(1);
    let parsed = match part_1 {
        Some(part) => part.splitn(2, ">")
                          .nth(1)
                          .unwrap()
                          .splitn(2, close)
                          .nth(0),
        None => None
    };
    match parsed {
        Some(s) => s,
        None => ""
    }
}
//Option<regex::Captures<'_>>

fn search_for_meta<'a>(meta: &'a str, content: &'a str) -> Option<regex::Captures<'a>>
{
    let pattern = format!(r#"<meta\s+name\s*=\s*"{}"\s+content\s*=\s*"([^"]*)"\s*/*>"#, meta);
    println!("{}", pattern);
    let re = Regex::new(&pattern).unwrap();
    re.captures(content)
}

#[tokio::test]
async fn check_new(){
    let metatag = Metatag::new("https://atareao.es").await.unwrap();
    println!("atareao: {}", metatag);
}
