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
		let bucket = Bucket::create(res, &bucket_uuid, 9).await.unwrap();
		assert!(path.join(&bucket_uuid).exists());

		fs::remove_dir_all(&path).await.is_err();
	}

	#[tokio::test]
	async fn test_create_failed() {
		let path = self::prepare_test().await.unwrap();

		let res = Store::create(&path, 1000, None).await.unwrap();
		assert!(path.exists());

		let bucket_uuid = Uuid::new_v4().to_string();
		let bucket = Bucket::create(res, &bucket_uuid, 1001).await.is_err();
		assert!(bucket);

		fs::remove_dir_all(&path).await.is_err();
	}

	#[tokio::test]
	async fn test_open() {
		let path = self::prepare_test().await.unwrap();

		let res = Store::create(&path, 1000, None).await.unwrap();
		assert!(path.exists());

		let bucket_uuid = Uuid::new_v4().to_string();
		let bucket_store = res.clone();

		let bucket = Bucket::create(bucket_store, &bucket_uuid, 999).await.unwrap();
		assert!(path.join(&bucket_uuid).exists());

		let bucket = Bucket::open(res, &bucket_uuid).await.unwrap();

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
		let mut bucket = Bucket::create(res, &bucket_uuid, 30).await.unwrap();
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
		let mut bucket = Bucket::create(res, &bucket_uuid, 9).await.unwrap();
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
		let mut bucket = Bucket::create(res, &bucket_uuid, 999).await.unwrap();
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
		let mut bucket = Bucket::create(res, &bucket_uuid, 999).await.unwrap();
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
		let mut bucket = Bucket::create(res, &bucket_uuid, 999).await.unwrap();
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
		let mut bucket = Bucket::create(res, &bucket_uuid, 999).await.unwrap();
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
		let mut bucket = Bucket::create(res, &bucket_uuid, 9).await.unwrap();
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
		let mut bucket = Bucket::create(res, &bucket_uuid, 9).await.unwrap();
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
		let mut bucket = Bucket::create(res, &bucket_uuid, 999).await.unwrap();
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

	#[tokio::test]
	async fn test_rename_files() {
		let path = self::prepare_test().await.unwrap();

		let res = Store::create(&path, 1000, None).await.unwrap();
		assert!(path.exists());

		let bucket_uuid = Uuid::new_v4().to_string();
		let mut bucket = Bucket::create(res, &bucket_uuid, 999).await.unwrap();
		assert!(path.join(&bucket_uuid).exists());

		let key = bucket.add(&KeyValue { key: None, name: "index".to_string(), value: None }).await.unwrap();
		assert!(path.join(&bucket_uuid).join("files").join("index").exists());

		let key_2 = bucket.add(&KeyValue { key: Some(key.to_string()), name: "index".to_string(), value: None }).await.unwrap();
		assert!(path.join(&bucket_uuid).join("files").join("index").join("index").exists());
		bucket.add(&KeyValue { key: Some(key.to_string()), name: "index.js".to_string(), value: Some(b"console.log(\"Hello world!\")".to_vec()) }).await.unwrap();
		let index_key = bucket.add(&KeyValue { key: Some(key_2), name: "index.js".to_string(), value: Some(b"console.log(\"Hello world!\")".to_vec()) }).await.unwrap();

		let hashes = bucket.rename(&index_key, &"test.js".to_string()).await.unwrap();
		assert_eq!(hashes.len(), 1);
		assert_ne!(hashes[0], index_key);

		fs::remove_dir_all(path).await.is_err();
	}

	#[tokio::test]
	async fn test_rename_dir() {
		let path = self::prepare_test().await.unwrap();

		let res = Store::create(&path, 1000, None).await.unwrap();
		assert!(path.exists());

		let bucket_uuid = Uuid::new_v4().to_string();
		let mut bucket = Bucket::create(res, &bucket_uuid, 999).await.unwrap();
		assert!(path.join(&bucket_uuid).exists());

		let new_key = bucket.add(&KeyValue { key: None, name: "new".to_string(), value: None }).await.unwrap();
		bucket.add(&KeyValue { key: Some(new_key), name: "index".to_string(), value: None }).await.unwrap();

		let key = bucket.add(&KeyValue { key: None, name: "index".to_string(), value: None }).await.unwrap();
		assert!(path.join(&bucket_uuid).join("files").join("index").exists());

		let key_2 = bucket.add(&KeyValue { key: Some(key.to_string()), name: "index".to_string(), value: None }).await.unwrap();
		assert!(path.join(&bucket_uuid).join("files").join("index").join("index").exists());
		bucket.add(&KeyValue { key: Some(key.to_string()), name: "index.js".to_string(), value: Some(b"console.log(\"Hello world!\")".to_vec()) }).await.unwrap();
		let index_key = bucket.add(&KeyValue { key: Some(key_2), name: "index.js".to_string(), value: Some(b"console.log(\"Hello world!\")".to_vec()) }).await.unwrap();

		let hashes = bucket.rename(&key, &"test".to_string()).await.unwrap();
		assert_eq!(hashes.len(), 4);

		fs::remove_dir_all(path).await.is_err();
	}

	#[tokio::test]
	async fn test_move_dir() {
		let path = self::prepare_test().await.unwrap();

		let res = Store::create(&path, 1000, None).await.unwrap();
		assert!(path.exists());

		let bucket_uuid = Uuid::new_v4().to_string();
		let mut bucket = Bucket::create(res, &bucket_uuid, 999).await.unwrap();
		assert!(path.join(&bucket_uuid).exists());

		let new_key = bucket.add(&KeyValue { key: None, name: "new".to_string(), value: None }).await.unwrap();
		bucket.add(&KeyValue { key: Some(new_key.clone()), name: "year".to_string(), value: None }).await.unwrap();
		bucket.add(&KeyValue { key: None, name: "new_name".to_string(), value: None }).await.unwrap();

		let key = bucket.add(&KeyValue { key: None, name: "dir_from".to_string(), value: None }).await.unwrap();
		assert!(path.join(&bucket_uuid).join("files").join("dir_from").exists());

		let key_2 = bucket.add(&KeyValue { key: Some(key.to_string()), name: "dir_from_child".to_string(), value: None }).await.unwrap();
		assert!(path.join(&bucket_uuid).join("files").join("dir_from").join("dir_from_child").exists());

		bucket.add(&KeyValue { key: Some(new_key.to_string()), name: "index.js".to_string(), value: Some(b"console.log(\"Hello world!\")".to_vec()) }).await.unwrap();
		let index_key = bucket.add(&KeyValue { key: Some(key_2.clone()), name: "test.js".to_string(), value: Some(b"console.log(\"Hello world!\")".to_vec()) }).await.unwrap();

		bucket.move_items(&new_key, &key_2).await.unwrap();

		// fs::remove_dir_all(path).await.is_err();
	}

	#[tokio::test]
	async fn test_move_file() {
		let path = self::prepare_test().await.unwrap();

		let res = Store::create(&path, 1000, None).await.unwrap();
		assert!(path.exists());

		let bucket_uuid = Uuid::new_v4().to_string();
		let mut bucket = Bucket::create(res, &bucket_uuid, 999).await.unwrap();
		assert!(path.join(&bucket_uuid).exists());

		let new_key = bucket.add(&KeyValue { key: None, name: "new".to_string(), value: None }).await.unwrap();
		bucket.add(&KeyValue { key: Some(new_key.clone()), name: "year".to_string(), value: None }).await.unwrap();

		let key = bucket.add(&KeyValue { key: None, name: "index".to_string(), value: None }).await.unwrap();
		assert!(path.join(&bucket_uuid).join("files").join("index").exists());

		let key_2 = bucket.add(&KeyValue { key: Some(key.to_string()), name: "index".to_string(), value: None }).await.unwrap();
		assert!(path.join(&bucket_uuid).join("files").join("index").join("index").exists());
		bucket.add(&KeyValue { key: Some(key.to_string()), name: "index.js".to_string(), value: Some(b"console.log(\"Hello world!\")".to_vec()) }).await.unwrap();
		let index_key = bucket.add(&KeyValue { key: Some(key_2), name: "test_1.js".to_string(), value: Some(b"console.log(\"Hello world!\")".to_vec()) }).await.unwrap();

		bucket.move_items(&index_key, &new_key).await.unwrap();

		fs::remove_dir_all(path).await.is_err();
	}

	#[tokio::test]
	async fn test_copy_dir() {
		let path = self::prepare_test().await.unwrap();

		let res = Store::create(&path, 1000, None).await.unwrap();
		assert!(path.exists());

		let bucket_uuid = Uuid::new_v4().to_string();
		let mut bucket = Bucket::create(res, &bucket_uuid, 999).await.unwrap();
		assert!(path.join(&bucket_uuid).exists());

		let new_key = bucket.add(&KeyValue { key: None, name: "new".to_string(), value: None }).await.unwrap();
		bucket.add(&KeyValue { key: Some(new_key.clone()), name: "year".to_string(), value: None }).await.unwrap();

		let key = bucket.add(&KeyValue { key: None, name: "dir_from".to_string(), value: None }).await.unwrap();
		assert!(path.join(&bucket_uuid).join("files").join("dir_from").exists());

		let key_2 = bucket.add(&KeyValue { key: Some(key.to_string()), name: "dir_from_child".to_string(), value: None }).await.unwrap();
		assert!(path.join(&bucket_uuid).join("files").join("dir_from").join("dir_from_child").exists());

		bucket.add(&KeyValue { key: Some(new_key.to_string()), name: "index.js".to_string(), value: Some(b"console.log(\"Hello world!\")".to_vec()) }).await.unwrap();
		let index_key = bucket.add(&KeyValue { key: Some(key_2.clone()), name: "test.js".to_string(), value: Some(b"console.log(\"Hello world!\")".to_vec()) }).await.unwrap();

		bucket.copy(&new_key, &key_2).await.unwrap();

		// fs::remove_dir_all(path).await.is_err();
	}

	#[tokio::test]
	async fn test_get_properties() {
		let path = self::prepare_test().await.unwrap();

		let res = Store::create(&path, 1000, None).await.unwrap();
		assert!(path.exists());

		let bucket_uuid = Uuid::new_v4().to_string();
		let mut bucket = Bucket::create(res, &bucket_uuid, 999).await.unwrap();
		assert!(path.join(&bucket_uuid).exists());

		let key = bucket.add(&KeyValue { key: None, name: "index.js".to_string(), value: Some(b"console.log(\"Hello world!\")".to_vec()) }).await.unwrap();
		assert!(path.join(&bucket_uuid).join("files").join("index.js").exists());

		let data = bucket.properties(&key).await.unwrap();

		assert_eq!(data.name, "index.js");
		assert_eq!(data.is_dir, false);
		assert_eq!(data.owner, 0);
		fs::remove_dir_all(path).await.is_err();
	}

	#[tokio::test]
	async fn test_set_favorite() {
		let path = self::prepare_test().await.unwrap();

		let res = Store::create(&path, 1000, None).await.unwrap();
		assert!(path.exists());

		let bucket_uuid = Uuid::new_v4().to_string();
		let mut bucket = Bucket::create(res, &bucket_uuid, 999).await.unwrap();
		assert!(path.join(&bucket_uuid).exists());

		let key = bucket.add(&KeyValue { key: None, name: "index.js".to_string(), value: Some(b"console.log(\"Hello world!\")".to_vec()) }).await.unwrap();
		assert!(path.join(&bucket_uuid).join("files").join("index.js").exists());

		bucket.set_favorites(vec![key.clone()]).await.unwrap();
		let data = bucket.properties(&key).await.unwrap();

		assert_eq!(data.name, "index.js");
		assert_eq!(data.is_dir, false);
		assert_eq!(data.is_favorite, true);
		fs::remove_dir_all(path).await.is_err();
	}

	#[tokio::test]
	async fn test_unset_favorite() {
		let path = self::prepare_test().await.unwrap();

		let res = Store::create(&path, 1000, None).await.unwrap();
		assert!(path.exists());

		let bucket_uuid = Uuid::new_v4().to_string();
		let mut bucket = Bucket::create(res, &bucket_uuid, 999).await.unwrap();
		assert!(path.join(&bucket_uuid).exists());

		let key = bucket.add(&KeyValue { key: None, name: "index.js".to_string(), value: Some(b"console.log(\"Hello world!\")".to_vec()) }).await.unwrap();
		assert!(path.join(&bucket_uuid).join("files").join("index.js").exists());

		bucket.set_favorites(vec![key.clone()]).await.unwrap();
		let data = bucket.properties(&key).await.unwrap();

		assert_eq!(data.name, "index.js");
		assert_eq!(data.is_dir, false);
		assert_eq!(data.is_favorite, true);

		bucket.unset_favorites(vec![key.clone()]).await.unwrap();
		let data = bucket.properties(&key).await.unwrap();

		assert_eq!(data.name, "index.js");
		assert_eq!(data.is_dir, false);
		assert_eq!(data.is_favorite, false);

		fs::remove_dir_all(path).await.is_err();
	}

	#[tokio::test]
	async fn test_get_favorites() {
		let path = self::prepare_test().await.unwrap();

		let res = Store::create(&path, 1000, None).await.unwrap();
		assert!(path.exists());

		let bucket_uuid = Uuid::new_v4().to_string();
		let mut bucket = Bucket::create(res, &bucket_uuid, 999).await.unwrap();
		assert!(path.join(&bucket_uuid).exists());

		let key = bucket.add(&KeyValue { key: None, name: "index.js".to_string(), value: Some(b"console.log(\"Hello world!\")".to_vec()) }).await.unwrap();
		assert!(path.join(&bucket_uuid).join("files").join("index.js").exists());

		let key_2 = bucket.add(&KeyValue { key: None, name: "index.html".to_string(), value: Some(b"console.log(\"Hello world!\")".to_vec()) }).await.unwrap();
		assert!(path.join(&bucket_uuid).join("files").join("index.html").exists());

		bucket.set_favorites(vec![key]).await.unwrap();
		bucket.set_favorites(vec![key_2]).await.unwrap();

		let data = bucket.get_favorites().await.unwrap();

		assert_eq!(data.len(), 2);

		fs::remove_dir_all(path).await.is_err();
	}

	#[tokio::test]
	async fn test_set_delete() {
		let path = self::prepare_test().await.unwrap();

		let res = Store::create(&path, 1000, None).await.unwrap();
		assert!(path.exists());

		let bucket_uuid = Uuid::new_v4().to_string();
		let mut bucket = Bucket::create(res, &bucket_uuid, 999).await.unwrap();
		assert!(path.join(&bucket_uuid).exists());

		let key = bucket.add(&KeyValue { key: None, name: "index.js".to_string(), value: Some(b"console.log(\"Hello world!\")".to_vec()) }).await.unwrap();
		assert!(path.join(&bucket_uuid).join("files").join("index.js").exists());

		bucket.set_delete(vec![key.clone()]).await.unwrap();
		let data = bucket.properties(&key).await.unwrap();

		assert_eq!(data.name, "index.js");
		assert_eq!(data.is_dir, false);
		assert_eq!(data.is_delete, true);

		fs::remove_dir_all(path).await.is_err();
	}

	#[tokio::test]
	async fn test_unset_delete() {
		let path = self::prepare_test().await.unwrap();

		let res = Store::create(&path, 1000, None).await.unwrap();
		assert!(path.exists());

		let bucket_uuid = Uuid::new_v4().to_string();
		let mut bucket = Bucket::create(res, &bucket_uuid, 999).await.unwrap();
		assert!(path.join(&bucket_uuid).exists());

		let key = bucket.add(&KeyValue { key: None, name: "index.js".to_string(), value: Some(b"console.log(\"Hello world!\")".to_vec()) }).await.unwrap();
		assert!(path.join(&bucket_uuid).join("files").join("index.js").exists());

		bucket.set_delete(vec![key.clone()]).await.unwrap();
		let data = bucket.properties(&key).await.unwrap();

		assert_eq!(data.name, "index.js");
		assert_eq!(data.is_dir, false);
		assert_eq!(data.is_delete, true);

		bucket.restore_delete(vec![key.clone()]).await.unwrap();
		let restored_data = bucket.properties(&key).await.unwrap();
		assert_eq!(restored_data.name, "index.js");
		assert_eq!(restored_data.is_dir, false);
		assert_eq!(restored_data.is_delete, false);

		fs::remove_dir_all(path).await.is_err();
	}

	#[tokio::test]
	async fn test_get_deletes() {
		let path = self::prepare_test().await.unwrap();

		let res = Store::create(&path, 1000, None).await.unwrap();
		assert!(path.exists());

		let bucket_uuid = Uuid::new_v4().to_string();
		let mut bucket = Bucket::create(res, &bucket_uuid, 999).await.unwrap();
		assert!(path.join(&bucket_uuid).exists());

		let key = bucket.add(&KeyValue { key: None, name: "index.js".to_string(), value: Some(b"console.log(\"Hello world!\")".to_vec()) }).await.unwrap();
		assert!(path.join(&bucket_uuid).join("files").join("index.js").exists());

		let key_2 = bucket.add(&KeyValue { key: None, name: "index.html".to_string(), value: Some(b"console.log(\"Hello world!\")".to_vec()) }).await.unwrap();
		assert!(path.join(&bucket_uuid).join("files").join("index.html").exists());

		bucket.set_delete(vec![key]).await.unwrap();
		bucket.set_delete(vec![key_2]).await.unwrap();

		let data = bucket.get_deletes().await.unwrap();

		assert_eq!(data.len(), 2);

		fs::remove_dir_all(path).await.is_err();
	}

	#[tokio::test]
	async fn test_get_path_deletes() {
		let path = self::prepare_test().await.unwrap();

		let res = Store::create(&path, 1000, None).await.unwrap();
		assert!(path.exists());

		let bucket_uuid = Uuid::new_v4().to_string();
		let mut bucket = Bucket::create(res, &bucket_uuid, 999).await.unwrap();
		assert!(path.join(&bucket_uuid).exists());

		let folder = bucket.add(&KeyValue { key: None, name: "folder".to_string(), value: None }).await.unwrap();
		assert!(path.join(&bucket_uuid).join("files").join("folder").exists());

		let key = bucket.add(&KeyValue { key: Some(folder), name: "index.js".to_string(), value: Some(b"console.log(\"Hello world!\")".to_vec()) }).await.unwrap();
		assert!(path.join(&bucket_uuid).join("files").join("folder").join("index.js").exists());

		let key_2 = bucket.add(&KeyValue { key: None, name: "index.html".to_string(), value: Some(b"console.log(\"Hello world!\")".to_vec()) }).await.unwrap();
		assert!(path.join(&bucket_uuid).join("files").join("index.html").exists());


		let data_1 = bucket.get_path(key).await.unwrap();
		let data_2 = bucket.get_path(key_2).await.unwrap();

		assert_eq!(data_2[0].title, "index.html");
		assert_eq!(data_1[0].title, "folder");
		assert_eq!(data_1[1].title, "index.js");

		fs::remove_dir_all(path).await.is_err();
	}

	#[tokio::test]
	async fn test_remove_deletes() {
		let path = self::prepare_test().await.unwrap();

		let res = Store::create(&path, 1000, None).await.unwrap();
		assert!(path.exists());

		let bucket_uuid = Uuid::new_v4().to_string();
		let mut bucket = Bucket::create(res, &bucket_uuid, 999).await.unwrap();
		assert!(path.join(&bucket_uuid).exists());

		let key = bucket.add(&KeyValue { key: None, name: "index.js".to_string(), value: Some(b"console.log(\"Hello world!\")".to_vec()) }).await.unwrap();
		assert!(path.join(&bucket_uuid).join("files").join("index.js").exists());

		let key_2 = bucket.add(&KeyValue { key: None, name: "index.html".to_string(), value: Some(b"console.log(\"Hello world!\")".to_vec()) }).await.unwrap();
		assert!(path.join(&bucket_uuid).join("files").join("index.html").exists());

		bucket.set_delete(vec![key.clone()]).await.unwrap();
		bucket.set_delete(vec![key_2]).await.unwrap();

		bucket.remove(vec![key.clone()]).await.unwrap();
		let data = bucket.get_deletes().await.unwrap();

		assert_eq!(data.len(), 1);

		fs::remove_dir_all(path).await.is_err();
	}

	#[tokio::test]
	async fn test_clear_trash_deletes() {
		let path = self::prepare_test().await.unwrap();

		let res = Store::create(&path, 1000, None).await.unwrap();
		assert!(path.exists());

		let bucket_uuid = Uuid::new_v4().to_string();
		let mut bucket = Bucket::create(res, &bucket_uuid, 999).await.unwrap();
		assert!(path.join(&bucket_uuid).exists());

		let key = bucket.add(&KeyValue { key: None, name: "index.js".to_string(), value: Some(b"console.log(\"Hello world!\")".to_vec()) }).await.unwrap();
		assert!(path.join(&bucket_uuid).join("files").join("index.js").exists());

		let key_2 = bucket.add(&KeyValue { key: None, name: "index.html".to_string(), value: Some(b"console.log(\"Hello world!\")".to_vec()) }).await.unwrap();
		assert!(path.join(&bucket_uuid).join("files").join("index.html").exists());

		bucket.set_delete(vec![key.clone()]).await.unwrap();
		bucket.set_delete(vec![key_2]).await.unwrap();

		bucket.clear_trash().await.unwrap();
		let data = bucket.get_deletes().await.unwrap();

		assert_eq!(data.len(), 0);

		fs::remove_dir_all(path).await.is_err();
	}
}
