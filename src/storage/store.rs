use std::io::Result;
use std::path::{Path, PathBuf};

use tokio::fs;

pub struct Store {
    pub uuid: String,
    pub store_path: String,
    pub store_name: String,
    pub available_space: u128,
    pub usage_space: u128,
    pub logging: bool,
}

impl Store {
    async fn create(path: impl AsRef<Path>, available_space: u128, logging: Option<bool>) -> Result<Store> {
        return Ok(Store {
            uuid: "".to_string(),
            store_path: "".to_string(),
            store_name: "".to_string(),
            available_space: 0,
            usage_space: 0,
            logging: false,
        });
    }
    
    async fn open(path: impl AsRef<Path>) -> Result<Store> {
        return Ok(Store {
            uuid: "".to_string(),
            store_path: "".to_string(),
            store_name: "".to_string(),
            available_space: 0,
            usage_space: 0,
            logging: false,
        });
    }
    
    async fn restore(path: impl AsRef<Path>) -> Result<Store> {
        return Ok(Store {
            uuid: "".to_string(),
            store_path: "".to_string(),
            store_name: "".to_string(),
            available_space: 0,
            usage_space: 0,
            logging: false,
        });
    }
}
