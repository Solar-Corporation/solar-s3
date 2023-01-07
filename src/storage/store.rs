use std::io::{Error, ErrorKind, Result};
use std::path::Path;
use std::str;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json;
use tokio::fs;
use uuid::Uuid;

use crate::storage::space::Space;

#[derive(Serialize, Deserialize)]
pub struct Store {
    pub uuid: String,
    pub store_path: String,
    pub store_name: String,
    pub available_space: u64,
    pub usage_space: u64,
    pub logging: bool,
}

#[async_trait(? Send)]
pub trait Storage {
    async fn create(path: impl AsRef<Path>, available_space: u64, logging: Option<bool>) -> Result<Store>;
    async fn open(path: impl AsRef<Path>) -> Result<Store>;
    async fn restore(&self) -> Result<Store>;
    async fn recalculation_usage_space(&self) -> Result<u64>;
}

#[async_trait(? Send)]
impl Storage for Store {
    async fn create(path: impl AsRef<Path>, available_space: u64, logging: Option<bool>) -> Result<Store> {
        let disk_space: u64 = Space::get_disc();
        let free_size: i128 = disk_space as i128 - available_space as i128;
        if free_size <= 0 {
            return Err(Error::new(ErrorKind::StorageFull, "There is no free space to initialize the Storage!"));
        }
        
        let path = Path::new(path.as_ref());
        fs::create_dir(&path).await?;
        
        let store = Store {
            uuid: Uuid::new_v4().to_string(),
            store_path: path.to_str().unwrap().to_string(),
            store_name: path.file_name().unwrap().to_str().unwrap().to_string(),
            available_space,
            usage_space: 0,
            logging: logging.unwrap_or(false),
        };
        
        let store_json = serde_json::to_string(&store)?;
    
        fs::write(&path.join("storage.json"), &store_json.as_bytes()).await?;
        return Ok(store);
    }
    
    async fn open(path: impl AsRef<Path>) -> Result<Store> {
        let path = Path::new(path.as_ref()).join("storage.json");
        let file = fs::read(&path).await?;
        let json_str = str::from_utf8(&file).unwrap();
        let storage: Store = serde_json::from_str(json_str)?;
        return Ok(storage);
    }
    
    async fn restore(&self) -> Result<Store> {
        return Ok(Store {
            uuid: "".to_string(),
            store_path: "".to_string(),
            store_name: "".to_string(),
            available_space: 0,
            usage_space: 0,
            logging: false,
        });
    }
    
    async fn recalculation_usage_space(&self) -> Result<u64> {
        return Ok(0);
    }
}
