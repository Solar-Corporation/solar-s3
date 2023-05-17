use std::hash::{Hash, Hasher};
use std::io::{Error, ErrorKind, Result};
use std::ops::Add;
use std::path::Path;

use tokio::fs;

use crate::bucket::bucket_db::{BucketDB, KeyPath};
use crate::bucket::fs_metadata::{FsItem, FsMetadata, PropertiesItem};
use crate::storage::store::{Storage, Store};

pub struct Bucket {
	pub uuid: String,
	pub path: String,
	pub store: Store,
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

pub struct PathItem {
	pub key: String,
	pub title: String,
}

impl Bucket {
	pub async fn create(store: Store, uuid: &str, bucket_space: u64) -> Result<Bucket> {
		if store.usage_space + bucket_space > store.available_space {
			return Err(Error::new(ErrorKind::StorageFull, "There is no free space to create the Bucket!"));
		}

		let path = Path::new(&store.store_path).join(&uuid);
		fs::create_dir(&path).await?;
		fs::create_dir(path.join("files")).await?;

		let fs_metadata = FsMetadata::new(&path).await?;
		fs_metadata.set_available_space(bucket_space).await;

		BucketDB::init(&path).await;

		return Ok(Bucket {
			uuid: uuid.to_string(),
			path: path.to_str().unwrap().to_string(),
			store,
			available_space: bucket_space,
			usage_space: 0,
		});
	}

	pub async fn open(store: Store, uuid: &str) -> Result<Bucket> {
		let path = Path::new(&store.store_path).join(&uuid);
		let fs_metadata = FsMetadata::new(&path).await?;
		let space = fs_metadata.get_space().await.unwrap();
		return Ok(Bucket {
			uuid: uuid.to_string(),
			path: path.to_str().unwrap().to_string(),
			store,
			available_space: space.available_space,
			usage_space: space.usage_space,
		});
	}

	pub async fn add(&mut self, key_value: &KeyValue) -> Result<String> {
		let mut connection = BucketDB::open(self.path.as_str()).await.unwrap();
		let transaction = connection.transaction().unwrap();

		let path = match &key_value.key {
			None => Path::new("").join(&key_value.name),
			Some(key) => {
				let dir_path = BucketDB::get_path(key.as_ref(), &transaction).await.unwrap();
				Path::new(&dir_path).join(&key_value.name)
			},
		};

		let path = match &key_value.value {
			None => format!("{}/", path.to_str().unwrap()),
			Some(_) => format!("{}", path.to_str().unwrap())
		};

		let save_path = Path::new(self.path.as_str()).join("files").join(&path);
		let hash = FsMetadata::calculate_hash(path.as_str());

		let key_path = &KeyPath {
			key: hash.to_string(),
			path,
			is_dir: key_value.value.is_none(),
		};

		BucketDB::add_key(key_path, &transaction).await;
		transaction.commit();

		match &key_value.value {
			None => fs::create_dir(&save_path).await?,
			Some(buffer) => {
				let bucket = FsMetadata::new(&self.path).await.unwrap();
				let file_size = buffer.len() as u64;
				bucket.increase_size(file_size).await.unwrap();
				self.store.update_space(file_size).await.unwrap();
				fs::write(&save_path, buffer).await?
			}
		}

		return Ok(hash);
	}

	pub async fn adds(&mut self, key_values: Vec<KeyValue>) -> Result<Vec<String>> {
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

		let path = BucketDB::get_path(key, &transaction).await.unwrap();
		let path = Path::new(&self.path).join("files").join(path);

		let fs_metadata = FsMetadata::new(&path).await?;
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
			let fs_metadata = FsMetadata::new(&path).await?;
			let fs_item = fs_metadata.info().await.unwrap();
			if fs_item.is_delete {
				continue;
			}
			dir_items.push(fs_item);
		}
		transaction.commit();

		return Ok(dir_items);
	}

	pub async fn rename(&self, key: &String, new_name: &String) -> Result<Vec<String>> {
		let mut connection = BucketDB::open(self.path.as_str()).await.unwrap();
		let transaction = connection.transaction().unwrap();

		let old_path = BucketDB::get_path(key, &transaction).await.unwrap();

		let name = Path::new(old_path.as_str()).file_name().unwrap().to_str().unwrap();
		let new_path = old_path.replace(name, new_name);

		let new_hashes = BucketDB::update_paths(old_path.as_str(), new_path.as_str(), &transaction).await.unwrap();

		let old_path_system = Path::new(&self.path).join("files").join(old_path.as_str());
		let new_path_system = Path::new(&self.path).join("files").join(new_path.as_str());

		fs::rename(old_path_system, new_path_system).await.unwrap();
		transaction.commit();

		return Ok(new_hashes);
	}

	pub async fn move_items(&self, key_from: &String, key_to: &String) -> Result<()> {
		if key_from.as_str() == key_to.as_str() {
			return Err(Error::new(ErrorKind::InvalidInput, "Keys must not match!"));
		}

		let mut connection = BucketDB::open(self.path.as_str()).await.unwrap();
		let transaction = connection.transaction().unwrap();

		let path_from = BucketDB::get_path(key_from, &transaction).await.unwrap();
		let path_to = BucketDB::get_path(key_to, &transaction).await.unwrap();

		if path_to.replacen(path_from.as_str(), "", 1).len() < path_to.len() {
			return Err(Error::new(ErrorKind::InvalidInput, "Path error!"));
		}

		let path_from = Path::new(path_from.as_str());
		let path_to = Path::new(path_to.as_str()).join(path_from.file_name().unwrap());

		BucketDB::update_paths(path_from.to_str().unwrap(), path_to.to_str().unwrap(), &transaction).await.unwrap();

		let path_from = Path::new(&self.path).join("files").join(path_from);
		let path_to = Path::new(&self.path).join("files").join(path_to);

		let fs_metadata = FsMetadata::new(path_from).await.unwrap();
		fs_metadata.move_path(path_to, true).await.unwrap();

		transaction.commit();
		return Ok(());
	}

	pub async fn copy(&self, key_from: &String, key_to: &String) -> Result<Vec<String>> {
		if key_from == key_to {
			return Err(Error::new(ErrorKind::InvalidInput, "Keys must not match!"));
		}

		let mut connection = BucketDB::open(self.path.as_str()).await.unwrap();
		let transaction = connection.transaction().unwrap();

		let path_from = BucketDB::get_path(key_from, &transaction).await.unwrap();
		let path_to = BucketDB::get_path(key_to, &transaction).await.unwrap();

		let path_from = Path::new(path_from.as_str());
		let path_to = match path_from.is_dir() {
			false => Path::new(path_to.as_str()).join(path_from.file_name().unwrap()).to_str().unwrap().to_string(),
			true => format!("{}/", Path::new(path_to.as_str()).join(path_from.file_name().unwrap()).to_str().unwrap().to_string()),
		};

		let hashes = BucketDB::copy_paths(path_from.to_str().unwrap(), path_to.as_str(), &transaction).await.unwrap();

		let path_from = Path::new(&self.path).join("files").join(path_from);
		let path_to = Path::new(&self.path).join("files").join(path_to);

		let fs_metadata = FsMetadata::new(path_from).await.unwrap();
		fs_metadata.move_path(&path_to, false).await.unwrap();

		transaction.commit();
		return Ok(hashes);
	}

	pub async fn properties(&self, key: &String) -> Result<PropertiesItem> {
		let mut connection = BucketDB::open(self.path.as_str()).await.unwrap();
		let transaction = connection.transaction().unwrap();

		let path = BucketDB::get_path(key, &transaction).await.unwrap();
		let path = Path::new(&self.path).join("files").join(path);

		let fs_metadata = FsMetadata::new(path).await.unwrap();
		let properties = fs_metadata.get_properties().await.unwrap();
		transaction.commit();

		return Ok(properties);
	}

	pub async fn set_favorites(&self, keys: Vec<String>) -> Result<()> {
		let mut connection = BucketDB::open(self.path.as_str()).await.unwrap();
		let transaction = connection.transaction().unwrap();

		for key in keys {
			let path = BucketDB::get_path(&key, &transaction).await.unwrap();
			let path = Path::new(&self.path).join("files").join(path);

			let fs_metadata = FsMetadata::new(path).await.unwrap();
			BucketDB::set_favorite(&key, &transaction).await.unwrap();
			fs_metadata.set_favorite().await.unwrap();
		}

		transaction.commit();

		return Ok(());
	}

	pub async fn unset_favorites(&self, keys: Vec<String>) -> Result<()> {
		let mut connection = BucketDB::open(self.path.as_str()).await.unwrap();
		let transaction = connection.transaction().unwrap();

		for key in keys {
			let path = BucketDB::get_path(&key, &transaction).await.unwrap();
			let path = Path::new(&self.path).join("files").join(path);

			let fs_metadata = FsMetadata::new(path).await.unwrap();
			BucketDB::unset_favorite(&key, &transaction).await.unwrap();
			fs_metadata.unset_favorite().await.unwrap();
		}

		return Ok(());
	}

	pub async fn get_favorites(&self) -> Result<Vec<FsItem>> {
		let mut connection = BucketDB::open(self.path.as_str()).await.unwrap();
		let transaction = connection.transaction().unwrap();

		let paths = BucketDB::get_favorites(&transaction).await.unwrap();

		let mut dir_items: Vec<FsItem> = Vec::new();
		for path in paths {
			let path = Path::new(&self.path).join("files").join(path);
			let fs_metadata = FsMetadata::new(&path).await?;
			let fs_item = fs_metadata.info().await.unwrap();
			if fs_item.is_delete {
				continue;
			}
			dir_items.push(fs_item);
		}

		transaction.commit();

		return Ok(dir_items);
	}

	pub async fn set_delete(&self, keys: Vec<String>) -> Result<()> {
		let mut connection = BucketDB::open(self.path.as_str()).await.unwrap();
		let transaction = connection.transaction().unwrap();

		for key in keys {
			let path = BucketDB::get_path(&key, &transaction).await.unwrap();
			let path = Path::new(&self.path).join("files").join(path);

			let fs_metadata = FsMetadata::new(path).await.unwrap();
			let timestamp = fs_metadata.set_delete().await.unwrap();

			BucketDB::set_delete(&key, timestamp + 2592000, &transaction).await.unwrap();
		}

		transaction.commit();

		return Ok(());
	}

	pub async fn restore_delete(&self, keys: Vec<String>) -> Result<()> {
		let mut connection = BucketDB::open(self.path.as_str()).await.unwrap();
		let transaction = connection.transaction().unwrap();

		for key in keys {
			let path = BucketDB::get_path(&key, &transaction).await.unwrap();
			let path = Path::new(&self.path).join("files").join(path);

			let fs_metadata = FsMetadata::new(path).await.unwrap();
			fs_metadata.restore_delete().await.unwrap();

			BucketDB::restore_delete(&key, &transaction).await.unwrap();
		}

		transaction.commit();

		return Ok(());
	}

	pub async fn get_deletes(&self) -> Result<Vec<FsItem>> {
		let mut connection = BucketDB::open(self.path.as_str()).await.unwrap();
		let transaction = connection.transaction().unwrap();

		let paths = BucketDB::get_deletes(&transaction).await.unwrap();

		let mut dir_items: Vec<FsItem> = Vec::new();
		for path in paths {
			let path = Path::new(&self.path).join("files").join(path);
			let fs_metadata = FsMetadata::new(&path).await?;
			let fs_item = fs_metadata.info().await.unwrap();
			dir_items.push(fs_item);
		}

		transaction.commit();

		return Ok(dir_items);
	}

	pub async fn get_path(&self, key: String) -> Result<Vec<PathItem>> {
		let mut connection = BucketDB::open(self.path.as_str()).await.unwrap();
		let transaction = connection.transaction().unwrap();

		let path = BucketDB::get_path(&key, &transaction).await.unwrap();
		let paths = path.split("/");

		let mut result: Vec<PathItem> = vec![];
		let mut path_to_hash: Vec<String> = vec![];

		for path in paths.into_iter() {
			path_to_hash.push(path.to_string());
			let path_hash = Path::new(&self.path).join("files").join(&path);
			let hash = FsMetadata::calculate_hash(path_hash.to_str().unwrap());

			result.push(PathItem {
				key: hash,
				title: path.to_string(),
			});
		}

		transaction.commit();
		return Ok(result);
	}

	pub async fn remove(&self, keys: Vec<String>) -> Result<()> {
		let mut connection = BucketDB::open(self.path.as_str()).await.unwrap();
		let transaction = connection.transaction().unwrap();

		for key in keys {
			let path = BucketDB::get_path(&key, &transaction).await.unwrap();
			let path = Path::new(&self.path).join("files").join(path);

			let fs_metadata = FsMetadata::new(path).await.unwrap();
			let file_prop = fs_metadata.get_properties().await.unwrap();
			BucketDB::remove_trash(&key, &transaction).await.unwrap();
			fs_metadata.remove().await.unwrap();
			fs_metadata.decrease_size(file_prop.size);
		}
		transaction.commit();

		return Ok(());
	}

	pub async fn clear_trash(&self) -> Result<()> {
		let mut connection = BucketDB::open(self.path.as_str()).await.unwrap();
		let transaction = connection.transaction().unwrap();
		let paths = BucketDB::clear_trash(&transaction).await.unwrap();

		for path in paths {
			let path = Path::new(&self.path).join("files").join(path);
			let fs_metadata = FsMetadata::new(path).await.unwrap();
			let file_prop = fs_metadata.get_properties().await.unwrap();
			fs_metadata.remove().await.unwrap();
			fs_metadata.decrease_size(file_prop.size);
		}

		transaction.commit();

		return Ok(());
	}

	pub async fn get_space() -> Result<String> {
		return Ok("test".to_string());
	}
}
