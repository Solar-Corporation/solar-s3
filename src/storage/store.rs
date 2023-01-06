use std::io::{Error, ErrorKind, Result};
use std::path::Path;
use std::str;

use async_trait::async_trait;
use mockall::*;
use serde::{Deserialize, Serialize};
use serde_json;
use tokio::{fs, io::AsyncWriteExt};
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
        
        let mut store_file = fs::File::create(&path.join("storage.json")).await?;
        store_file.write_all(&store_json.as_bytes()).await?;
        
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

#[cfg(test)]
mod tests_store {
    use mocktopus::mocking::*;
    
    use crate::storage::space::Space;
    
    use super::*;
    
    #[tokio::test]
    async fn test_create_success() {
        let path = Path::new("../storage_create");
        fs::remove_dir_all(path).await.is_err();
        let res = Store::create(&path, 1000, None).await.unwrap();
        assert!(path.exists());
        fs::remove_dir_all(path).await.is_err();
    }
    
    #[tokio::test]
    async fn test_create_check_json() {
        let path = Path::new("../storage_check_json");
        fs::remove_dir_all(path).await.is_err();
        let res = Store::create(&path, 1000, None).await.unwrap();
        assert!(path.join("storage.json").exists());
        fs::remove_dir_all(path).await.is_err();
    }
    
    #[tokio::test]
    async fn test_create_failed() {
        Space::get_disc.mock_safe(|| MockResult::Return(0));
        let path = Path::new("../storage_failed");
        fs::remove_dir_all(path).await.is_err();
        let res = Store::create(&path, 10, None).await.is_err();
        assert!(res);
    }
    
    #[tokio::test]
    async fn test_open() {
        Space::get_disc.mock_safe(|| MockResult::Return(10000));
        let path = Path::new("../storage_open");
        fs::remove_dir_all(path).await.is_err();
        
        let res = Store::create(&path, 1000, None).await.unwrap();
        assert!(path.exists());
        
        let res = Store::open(path).await.unwrap();
        let is_uuid = Uuid::parse_str(res.uuid.as_str()).is_err();
        
        assert_eq!(res.store_path, "../storage_open");
        assert_eq!(is_uuid, false);
        assert_eq!(res.store_name, "storage_open");
        assert_eq!(res.logging, false);
        assert_eq!(res.available_space, 1000);
        assert_eq!(res.usage_space, 0);
        fs::remove_dir_all(path).await.is_err();
    }
}
