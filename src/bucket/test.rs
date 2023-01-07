#[cfg(test)]
mod tests_bucket {
	use std::io::Result;
	use std::path::{Path, PathBuf};
	
	use tokio::fs;
	use uuid::Uuid;
	
	use crate::bucket::bucket::{Bucket, KeyValue};
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
		let path = self::prepare_test().await.unwrap();
		
		let res = Store::create(&path, 1000, None).await.unwrap();
		assert!(path.exists());
		
		let bucket_uuid = Uuid::new_v4().to_string();
		let bucket = Bucket::create(&res, &bucket_uuid, 9).await.unwrap();
		assert!(path.join(&bucket_uuid).exists());
		
		fs::remove_dir_all(&path).await.is_err();
	}
	
	#[tokio::test]
	async fn test_create_failed() {
		let path = self::prepare_test().await.unwrap();
		
		let res = Store::create(&path, 1000, None).await.unwrap();
		assert!(path.exists());
		
		let bucket_uuid = Uuid::new_v4().to_string();
		let bucket = Bucket::create(&res, &bucket_uuid, 1001).await.is_err();
		assert!(bucket);
		
		fs::remove_dir_all(&path).await.is_err();
	}
	
	#[tokio::test]
	async fn test_open() {
		let path = self::prepare_test().await.unwrap();
		
		let res = Store::create(&path, 1000, None).await.unwrap();
		assert!(path.exists());
		
		let bucket_uuid = Uuid::new_v4().to_string();
		let bucket = Bucket::create(&res, &bucket_uuid, 999).await.unwrap();
		assert!(path.join(&bucket_uuid).exists());
		
		let bucket = Bucket::open(&res, &bucket_uuid).await.unwrap();
		
		assert_eq!(bucket.usage_space, 0);
		assert_eq!(bucket.path, path.join(&bucket_uuid).to_str().unwrap().to_string());
		assert_eq!(bucket.available_space, 999);
		
		fs::remove_dir_all(path).await.is_err();
	}
	
	#[tokio::test]
	async fn test_add_file() {
		let path = self::prepare_test().await.unwrap();
		
		let res = Store::create(&path, 1000, None).await.unwrap();
		assert!(path.exists());
		
		let bucket_uuid = Uuid::new_v4().to_string();
		let mut bucket = Bucket::create(&res, &bucket_uuid, 9).await.unwrap();
		assert!(path.join(&bucket_uuid).exists());
		
		bucket.add(&KeyValue { key: None, name: "index.js".to_string(), value: Some(b"console.log(\"Hello world!\")".to_vec()) }).await;
		assert!(path.join(&bucket_uuid).join("files").join("index.js").exists());
		
		fs::remove_dir_all(path).await.is_err();
	}
	
	#[tokio::test]
	async fn test_add_dir() {
		let path = self::prepare_test().await.unwrap();
		
		let res = Store::create(&path, 1000, None).await.unwrap();
		assert!(path.exists());
		
		let bucket_uuid = Uuid::new_v4().to_string();
		let mut bucket = Bucket::create(&res, &bucket_uuid, 9).await.unwrap();
		assert!(path.join(&bucket_uuid).exists());
		
		bucket.add(&KeyValue { key: None, name: "index".to_string(), value: None }).await;
		assert!(path.join(&bucket_uuid).join("files").join("index").exists());
		
		fs::remove_dir_all(path).await.is_err();
	}
	
	#[tokio::test]
	async fn test_add_in_dir() {
		let path = self::prepare_test().await.unwrap();
		
		let res = Store::create(&path, 1000, None).await.unwrap();
		assert!(path.exists());
		
		let bucket_uuid = Uuid::new_v4().to_string();
		let mut bucket = Bucket::create(&res, &bucket_uuid, 9).await.unwrap();
		assert!(path.join(&bucket_uuid).exists());
		
		let key = bucket.add(&KeyValue { key: None, name: "index".to_string(), value: None }).await.unwrap();
		assert!(path.join(&bucket_uuid).join("files").join("index").exists());
		
		let key_2 = bucket.add(&KeyValue { key: Some(key.to_string()), name: "index".to_string(), value: None }).await.unwrap();
		assert!(path.join(&bucket_uuid).join("files").join("index").join("index").exists());
		bucket.add(&KeyValue { key: Some(key.to_string()), name: "index.js".to_string(), value: Some(b"console.log(\"Hello world!\")".to_vec()) }).await;
		bucket.add(&KeyValue { key: Some(key_2), name: "index.js".to_string(), value: Some(b"console.log(\"Hello world!\")".to_vec()) }).await;
		
		fs::remove_dir_all(path).await.is_err();
	}
	
	
	#[tokio::test]
	async fn test_adds() {
		let path = self::prepare_test().await.unwrap();
		
		let res = Store::create(&path, 1000, None).await.unwrap();
		assert!(path.exists());
		
		let bucket_uuid = Uuid::new_v4().to_string();
		let mut bucket = Bucket::create(&res, &bucket_uuid, 9).await.unwrap();
		assert!(path.join(&bucket_uuid).exists());
		
		let key = bucket.add(&KeyValue { key: None, name: "index".to_string(), value: None }).await.unwrap();
		assert!(path.join(&bucket_uuid).join("files").join("index").exists());
		
		let key_values = vec![
			KeyValue { key: Some(key.to_string()), name: "index".to_string(), value: None },
			KeyValue {
				key: Some(key.to_string()),
				name: "index.js".to_string(),
				value: Some(b"console.log(\"Hello world!\")".to_vec()),
			}];
		bucket.adds(key_values).await;
		assert!(path.join(&bucket_uuid).join("files").join("index").join("index").exists());
		assert!(path.join(&bucket_uuid).join("files").join("index").join("index.js").exists());
		
		fs::remove_dir_all(path).await.is_err();
	}
	
	#[tokio::test]
	async fn test_get_file() {
		let path = self::prepare_test().await.unwrap();
		
		let res = Store::create(&path, 1000, None).await.unwrap();
		assert!(path.exists());
		
		let bucket_uuid = Uuid::new_v4().to_string();
		let mut bucket = Bucket::create(&res, &bucket_uuid, 9).await.unwrap();
		assert!(path.join(&bucket_uuid).exists());
		
		let key = bucket.add(&KeyValue { key: None, name: "index.js".to_string(), value: Some(b"console.log(\"Hello world!\")".to_vec()) }).await.unwrap();
		assert!(path.join(&bucket_uuid).join("files").join("index.js").exists());
		
		let data = bucket.get(&key, true).await.unwrap();
		
		assert_eq!(data.name, "index.js");
		fs::remove_dir_all(path).await.is_err();
	}
	
	#[tokio::test]
	async fn test_get_file_buffer() {
		let path = self::prepare_test().await.unwrap();
		
		let res = Store::create(&path, 1000, None).await.unwrap();
		assert!(path.exists());
		
		let bucket_uuid = Uuid::new_v4().to_string();
		let mut bucket = Bucket::create(&res, &bucket_uuid, 9).await.unwrap();
		assert!(path.join(&bucket_uuid).exists());
		
		let key = bucket.add(&KeyValue { key: None, name: "index.js".to_string(), value: Some(b"console.log(\"Hello world!\")".to_vec()) }).await.unwrap();
		assert!(path.join(&bucket_uuid).join("files").join("index.js").exists());
		
		let data = bucket.get(&key, false).await.unwrap();
		
		assert_eq!(&data.name, "index.js");
		assert_eq!(&data.buffer.unwrap(), b"console.log(\"Hello world!\")");
		fs::remove_dir_all(path).await.is_err();
	}
	
	#[tokio::test]
	async fn test_get_dir() {
		let path = self::prepare_test().await.unwrap();
		
		let res = Store::create(&path, 1000, None).await.unwrap();
		assert!(path.exists());
		
		let bucket_uuid = Uuid::new_v4().to_string();
		let mut bucket = Bucket::create(&res, &bucket_uuid, 9).await.unwrap();
		assert!(path.join(&bucket_uuid).exists());
		
		let key = bucket.add(&KeyValue { key: None, name: "index".to_string(), value: None }).await.unwrap();
		assert!(path.join(&bucket_uuid).join("files").join("index").exists());
		
		let data = bucket.get(&key, true).await.unwrap();
		
		assert_eq!(&data.name, "index");
		fs::remove_dir_all(path).await.is_err();
	}
	
	#[tokio::test]
	async fn test_get_dir_err() {
		let path = self::prepare_test().await.unwrap();
		
		let res = Store::create(&path, 1000, None).await.unwrap();
		assert!(path.exists());
		
		let bucket_uuid = Uuid::new_v4().to_string();
		let mut bucket = Bucket::create(&res, &bucket_uuid, 9).await.unwrap();
		assert!(path.join(&bucket_uuid).exists());
		
		let key = bucket.add(&KeyValue { key: None, name: "index".to_string(), value: None }).await.unwrap();
		assert!(path.join(&bucket_uuid).join("files").join("index").exists());
		
		let data = bucket.get(&key, false).await.is_err();
		
		assert!(data);
		fs::remove_dir_all(path).await.is_err();
	}
	
	#[tokio::test]
	async fn test_get_dir_items() {
		let path = self::prepare_test().await.unwrap();
		
		let res = Store::create(&path, 1000, None).await.unwrap();
		assert!(path.exists());
		
		let bucket_uuid = Uuid::new_v4().to_string();
		let mut bucket = Bucket::create(&res, &bucket_uuid, 9).await.unwrap();
		assert!(path.join(&bucket_uuid).exists());
		let key = bucket.add(&KeyValue { key: None, name: "index".to_string(), value: None }).await.unwrap();
		assert!(path.join(&bucket_uuid).join("files").join("index").exists());
		
		let key_values = vec![
			KeyValue { key: Some(key.to_string()), name: "index.js".to_string(), value: Some(b"console.log(\"Hello world!\")".to_vec()) },
			KeyValue { key: Some(key.to_string()), name: "test.js".to_string(), value: Some(b"console.log(\"Hello world!\")".to_vec()) },
		];
		let keys = bucket.adds(key_values).await.unwrap();
		assert!(path.join(&bucket_uuid).join("files").join("index").exists());
		assert!(path.join(&bucket_uuid).join("files").join("index").join("index.js").exists());
		assert!(path.join(&bucket_uuid).join("files").join("index").join("test.js").exists());
		
		let data = bucket.get_items(None).await.unwrap();
		assert_eq!(data.len(), 1);
		assert_eq!(data[0].hash, "FD1EA89060210A4E");
		
		let data = bucket.get_items(Some(&key)).await.unwrap();
		assert_eq!(data.len(), 2);
		fs::remove_dir_all(path).await.is_err();
	}
}
