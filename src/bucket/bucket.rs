use std::hash::{Hash, Hasher};
use std::io::{Error, ErrorKind, Result};
use std::path::Path;

use tokio::fs;

use crate::bucket::bucket_db::{BucketDB, KeyPath};
use crate::bucket::fs_metadata::{FsItem, FsMetadata};
use crate::storage::store::Store;

pub struct Bucket {
	pub uuid: String,
	pub path: String,
	pub available_space: u64,
	pub usage_space: u64,
}

pub struct GetOptions {
	pub info_only: bool,
	pub get_delete: Option<bool>,
}

pub struct KeyValue {
	pub key: Option<String>,
	pub name: String,
	pub value: Option<Vec<u8>>,
}

impl Bucket {
	pub async fn create(store: &Store, uuid: &str, bucket_space: u64) -> Result<Bucket> {
		let free_size: i128 = store.available_space as i128 - store.usage_space as i128;
		
		if free_size < bucket_space as i128 {
			return Err(Error::new(ErrorKind::StorageFull, "There is no free space to create the Bucket!"));
		}
		
		let path = Path::new(&store.store_path).join(&uuid);
		fs::create_dir(&path).await?;
		fs::create_dir(path.join("files")).await?;
		
		let fs_metadata = FsMetadata::new(&path).unwrap();
		fs_metadata.set_available_space(bucket_space).await;
		
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
		let fs_metadata = FsMetadata::new(&path).unwrap();
		let space = fs_metadata.get_space().await.unwrap();
		return Ok(Bucket {
			uuid: uuid.to_string(),
			path: path.to_str().unwrap().to_string(),
			available_space: space.available_space,
			usage_space: space.usage_space,
		});
	}
	
	pub async fn add(&self, key_value: &KeyValue) -> Result<String> {
		let mut connection = BucketDB::open(self.path.as_str()).await.unwrap();
		let transaction = connection.transaction().unwrap();
		
		let path = match &key_value.key {
			None => Path::new("").join(&key_value.name),
			Some(key) => {
				let dir_path = BucketDB::get_path(key.as_ref(), &transaction).await.unwrap();
				Path::new(&dir_path).join(&key_value.name)
			},
		};
		
		let save_path = Path::new(self.path.as_str()).join("files").join(&path);
		let hash = FsMetadata::calculate_hash(path.to_str().unwrap());
		
		let key_path = KeyPath {
			key: hash.to_string(),
			path: path.to_str().unwrap().to_string(),
			is_dir: key_value.value.is_none(),
		};
		
		BucketDB::add_key(key_path, &transaction).await;
		transaction.commit();
		
		match &key_value.value {
			None => fs::create_dir(&save_path).await?,
			Some(buffer) => fs::write(&save_path, buffer).await?
		}
		
		return Ok(hash);
	}
	
	pub async fn adds(&self, key_values: Vec<KeyValue>) -> Result<Vec<String>> {
		let mut keys: Vec<String> = Vec::new();
		for key_value in key_values.iter() {
			let key = self.add(key_value).await.unwrap();
			keys.push(key);
		}
		
		return Ok(keys);
	}
	
	pub async fn get(&self, key: &String, info_only: bool) -> Result<FsItem> {
		let mut connection = BucketDB::open(self.path.as_str()).await.unwrap();
		let transaction = connection.transaction().unwrap();
		
		let path = BucketDB::get_path(key.as_ref(), &transaction).await.unwrap();
		let path = Path::new(&self.path).join("files").join(path);
		
		let fs_metadata = FsMetadata::new(&path).unwrap();
		let mut fs_item = fs_metadata.info().await.unwrap();
		
		if !info_only && fs_item.is_dir {
			return Err(Error::new(ErrorKind::IsADirectory, "Can't get data because it's a directory!"));
		}
		
		if !info_only {
			fs_item.buffer = Some(fs::read(&path).await?);
		}
		
		transaction.commit();
		return Ok(fs_item);
	}
	
	pub async fn get_items(&self, key: Option<&String>) -> Result<Vec<FsItem>> {
		let mut connection = BucketDB::open(self.path.as_str()).await.unwrap();
		let transaction = connection.transaction().unwrap();
		
		let path = match key {
			None => "".to_string(),
			Some(key) => BucketDB::get_path(key.as_ref(), &transaction).await.unwrap(),
		};
		
		let path = Path::new(&self.path).join("files").join(path);
		
		let mut dir_items: Vec<FsItem> = Vec::new();
		let mut dir = fs::read_dir(&path).await?;
		if !fs::metadata(&path).await?.is_dir() {
			return Err(Error::new(ErrorKind::NotADirectory, "This is not a directory!"));
		}
		
		while let Some(item) = dir.next_entry().await? {
			let path = &item.path();
			let fs_metadata = FsMetadata::new(&path).unwrap();
			let fs_item = fs_metadata.info().await.unwrap();
			if fs_item.is_delete {
				continue;
			}
			dir_items.push(fs_item);
		}
		
		return Ok(dir_items);
	}
}
