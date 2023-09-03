use serde::de::Error;
use serde_json;
use std::path::{Path, PathBuf};
use tokio::fs;

pub async fn load_config_file(file_path: PathBuf) -> serde_json::Result<serde_json::Value> {
    let content = fs::read_to_string(file_path).await;

    match content {
        Ok(content) => serde_json::from_str(&content),
        Err(e) => {
            println!("Error: {}", e);
            return Err(Error::custom("Failed to read config file"));
        }
    }
}
