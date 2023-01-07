#[cfg(test)]
mod tests_store {
	use std::io::Result;
	use std::path::{Path, PathBuf};
	
	use mocktopus::mocking::*;
	use tokio::fs;
	use uuid::Uuid;
	
	use crate::storage::space::Space;
	use crate::storage::store::{Storage, Store};
	
	async fn prepare_test() -> Result<PathBuf> {
		let path = Path::new("../storages");
		fs::create_dir(&path).await.is_err();
		let path = Path::new(&path).join(Uuid::new_v4().to_string());
		fs::remove_dir_all(&path).await.is_err();
		return Ok(path);
	}
	
	#[tokio::test]
	async fn test_create_success() {
		let path = prepare_test().await.unwrap();
		
		Store::create(&path, 1000, None).await.unwrap();
		assert!(path.exists());
		
		fs::remove_dir_all(&path).await.is_err();
	}
	
	#[tokio::test]
	async fn test_create_check_json() {
		let path = prepare_test().await.unwrap();
		
		Store::create(&path, 1000, None).await.unwrap();
		assert!(path.join("storage.json").exists());
		
		fs::remove_dir_all(path).await.is_err();
	}
	
	#[tokio::test]
	async fn test_create_failed() {
		Space::get_disc.mock_safe(|| MockResult::Return(0));
		let path = prepare_test().await.unwrap();
		
		let res = Store::create(&path, 10, None).await.is_err();
		assert!(res);
		
		fs::remove_dir_all(path).await.is_err();
	}
	
	#[tokio::test]
	async fn test_open() {
		Space::get_disc.mock_safe(|| MockResult::Return(10000));
		let path = prepare_test().await.unwrap();
		
		Store::create(&path, 1000, None).await.unwrap();
		assert!(path.exists());
		
		let res = Store::open(&path).await.unwrap();
		let is_uuid = Uuid::parse_str(res.uuid.as_str()).is_err();
		
		assert_eq!(res.store_path, path.to_str().unwrap().to_string());
		assert_eq!(is_uuid, false);
		assert_eq!(res.logging, false);
		assert_eq!(res.available_space, 1000);
		assert_eq!(res.usage_space, 0);
		fs::remove_dir_all(&path).await.is_err();
	}
}
