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
