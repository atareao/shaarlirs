use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    title: String,
    header_link: String,
    timezone: String,
    enabled_plugins: Vec<String>,
    default_private_links: bool,
}

impl Settings{
    pub fn new(title: &str, header_link: &str, timezone: &str, enabled_plugins: Vec<String>, default_private_links: bool) -> Self{
        Self{
            title: title.to_string(),
            header_link: header_link.to_string(),
            timezone: timezone.to_string(),
            enabled_plugins,
            default_private_links,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Info {
    global_counter: i64,
    private_counter: i64,
    settings: Settings,
}

impl Info{
    pub fn new(global_counter: i64, private_counter: i64, settings: Settings) -> Self{
        Self{
            global_counter,
            private_counter,
            settings,
        }
    }
}
