use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct Settings {
    title: String,
    header_link: String,
    timezone: String,
    enabled_plugins: Vec<String>,
    default_private_links: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Info {
    global_counter: i64,
    private_counter: i64,
}
