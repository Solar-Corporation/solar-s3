use std::io::{Error, ErrorKind, Result};
use std::path::Path;

use tokio::fs;

use crate::bucket::bucket_db::BucketDB;
use crate::bucket::fs_metadata::FsMetadata;
use crate::storage::store::Store;

pub struct Bucket {
	pub uuid: String,
	pub path: String,
	pub available_space: u64,
	pub usage_space: u64,
}

impl Bucket {
	pub async fn create(store: &Store, uuid: &str, bucket_space: u64) -> Result<Bucket> {
		let free_size: i128 = store.available_space as i128 - store.usage_space as i128;
		
		if free_size < bucket_space as i128 {
			return Err(Error::new(ErrorKind::StorageFull, "There is no free space to create the Bucket!"));
		}
		
		let path = Path::new(&store.store_path).join(&uuid);
		fs::create_dir(&path).await?;
		fs::create_dir(path.join("files"));
		
		FsMetadata::set_available_space(&path, bucket_space).await;
		
		BucketDB::init(&path).await;
		
		return Ok(Bucket {
			uuid: uuid.to_string(),
			path: path.to_str().unwrap().to_string(),
			available_space: bucket_space,
			usage_space: 0,
		});
	}
	
	pub async fn open(store: &Store, uuid: &str) -> Result<Bucket> {
		let path = Path::new(&store.store_path).join(&uuid);
		let space = FsMetadata::get_space(&path).await.unwrap();
		return Ok(Bucket {
			uuid: uuid.to_string(),
			path: path.to_str().unwrap().to_string(),
			available_space: space.available_space,
			usage_space: space.usage_space,
		});
	}
}

#[cfg(test)]
mod tests_bucket {
	use crate::storage::store::{Storage, Store};
	
	use super::*;
	
	#[tokio::test]
	async fn test_create_success() {
		let path = Path::new("../storage_create_init_bucket");
		fs::remove_dir_all(path).await.is_err();
		
		let res = Store::create(&path, 1000, None).await.unwrap();
		assert!(path.exists());
		
		let bucket = Bucket::create(&res, "test_bucket", 9).await.unwrap();
		assert!(path.join("test_bucket").exists());
		
		fs::remove_dir_all(path).await.is_err();
	}
	
	#[tokio::test]
	async fn test_create_failed() {
		let path = Path::new("../storage_create_init_bucket_failed");
		fs::remove_dir_all(&path).await.is_err();
		
		let res = Store::create(&path, 1000, None).await.unwrap();
		assert!(path.exists());
		
		let bucket = Bucket::create(&res, "test_bucket", 1001).await.is_err();
		assert!(bucket);
		fs::remove_dir_all(&path).await.is_err();
	}
	
	#[tokio::test]
	async fn test_open() {
		let path = Path::new("../storage_open_bucket");
		fs::remove_dir_all(path).await.is_err();
		let res = Store::create(&path, 1000, None).await.unwrap();
		assert!(path.exists());
		
		let bucket = Bucket::create(&res, "test_bucket", 999).await.unwrap();
		assert!(path.join("test_bucket").exists());
		
		let bucket = Bucket::open(&res, "test_bucket").await.unwrap();
		
		assert_eq!(bucket.usage_space, 0);
		assert_eq!(bucket.path, "../storage_open_bucket/test_bucket");
		assert_eq!(bucket.available_space, 999);
		
		fs::remove_dir_all(path).await.is_err();
	}
}
