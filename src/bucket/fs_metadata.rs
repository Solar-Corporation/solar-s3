use std::collections::hash_map::DefaultHasher;
use std::ffi::OsStr;
use std::hash::{Hash, Hasher};
use std::io::{Error, ErrorKind, Result};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::str;
use std::time::UNIX_EPOCH;

use byte_unit::Byte;
use chrono::Utc;
use futures::future::{BoxFuture, FutureExt};
use mime_guess::mime;
use regex::Regex;
use tokio::fs;
use xattr::{get, remove, set};

use crate::storage::space::Space;

pub struct Size {
	pub available_space: u64,
	pub usage_space: u64,
}

pub struct FsItem {
	pub name: String,
	pub hash: String,
	pub size: String,
	pub file_type: String,
	pub mime_type: String,
	pub is_dir: bool,
	pub is_favorite: bool,
	pub is_delete: bool,
	pub see_time: u64,
	pub delete_at: Option<u64>,
	pub buffer: Option<Vec<u8>>,
}

pub struct PropertiesItem {
	pub name: String,
	pub hash: String,
	pub is_dir: bool,
	pub owner: u64,
	pub create_at: u64,
	pub update_at: u64,
	pub see_time: u64,
	pub is_favorite: bool,
	pub description: String,
	pub is_delete: bool,
	pub size: u64,
}

#[derive(Debug)]
pub struct FsMetadata {
	path: PathBuf,
	base_path: String,
	pub is_dir: bool,
}

impl FsMetadata {
	pub async fn new(path: impl AsRef<Path>) -> Result<FsMetadata> {
		let path_regex = Regex::new(r".*([\da-f]{8}-[\da-f]{4}-[\da-f]{4}-[\da-f]{4}-[\da-f]{12})(/files/)").unwrap();
		let base_path = path_regex.replace(path.as_ref().to_str().unwrap(), "");

		return Ok(FsMetadata {
			path: path.as_ref().to_path_buf(),
			is_dir: fs::metadata(&path).await?.is_dir(),
			base_path: base_path.to_string(),
		});
	}

	pub async fn increase_size(&self, add_size: u64) -> Result<u64> {
		let current_size = &self.get_space().await.unwrap();

		let new_size = current_size.usage_space + add_size;
		if new_size > current_size.available_space {
			return Err(Error::new(ErrorKind::StorageFull, "There is no free space to Add file!"));
		}
		set(&self.path, "user.usage_space", new_size.to_string().as_bytes());

		return Ok(new_size);
	}

	pub async fn decrease_size(&self, delete_size: u64) -> Result<u64> {
		let current_size = &self.get_space().await.unwrap();

		let new_size = current_size.usage_space - delete_size;
		set(&self.path, "user.usage_space", new_size.to_string().as_bytes());
		return Ok(new_size);
	}

	pub async fn set_delete(&self) -> Result<i64> {
		let delete_at = Utc::now().timestamp();

		set(&self.path, "user.is_delete", "true".as_bytes())?;
		set(&self.path, "user.delete_time", delete_at.to_string().as_bytes())?;

		return Ok(delete_at);
	}

	pub async fn restore_delete(&self) -> Result<()> {
		remove(&self.path, "user.is_delete")?;
		remove(&self.path, "user.delete_time")?;
		return Ok(());
	}

	pub async fn get_delete_time(&self) -> Result<Option<i64>> {
		const IS_DELETE: i64 = 0;

		let metadata_delete_value = get(&self.path, "user.delete_time")
			.unwrap_or(Some(IS_DELETE.to_string().as_bytes().to_vec()))
			.unwrap_or(IS_DELETE.to_string().as_bytes().to_vec());

		let metadata_value = str::from_utf8(&metadata_delete_value)
			.unwrap()
			.parse::<i64>()
			.unwrap();

		if metadata_value != 0 {
			return Ok(Some(metadata_value));
		}

		return Ok(None);
	}

	pub async fn set_available_space(&self, available_space: u64) -> Result<()> {
		set(&self.path, "user.available_space", available_space.to_string().as_bytes());
		return Ok(());
	}

	pub async fn get_space(&self) -> Result<Size> {
		const AVAILABLE_SPACE: u64 = 0;
		const NONE_MSG: &str = "none_space";

		if !&self.is_dir {
			return Err(Error::new(ErrorKind::NotADirectory, "This is not a directory!"));
		}

		let path = PathBuf::from(&self.path);

		let available_space = get(&path, "user.available_space")
			.unwrap_or(Some(AVAILABLE_SPACE.to_string().as_bytes().to_vec()))
			.unwrap_or(AVAILABLE_SPACE.to_string().as_bytes().to_vec());
		let available_space = str::from_utf8(&available_space).unwrap();


		let usage_space = get(&path, "user.usage_space")
			.unwrap_or(Some(NONE_MSG.as_bytes().to_vec()))
			.unwrap_or(NONE_MSG.as_bytes().to_vec());
		let usage_space = str::from_utf8(&usage_space).unwrap();

		if usage_space == NONE_MSG {
			let size = Space::dir_size(&path).await;
			set(&path, "user.usage_space", size.to_string().as_bytes());
			return Ok(Size {
				available_space: available_space.parse::<u64>().unwrap(),
				usage_space: size,
			});
		}

		return Ok(Size {
			available_space: available_space.parse::<u64>().unwrap(),
			usage_space: usage_space.parse::<u64>().unwrap(),
		});
	}

	pub async fn is_delete(&self) -> Result<bool> {
		const IS_DELETE: &str = "false";

		let metadata_vec_value = get(&self.path, "user.is_delete")
			.unwrap_or(Some(IS_DELETE.as_bytes().to_vec()))
			.unwrap_or(IS_DELETE.as_bytes().to_vec());
		let metadata_value = str::from_utf8(&metadata_vec_value).unwrap();

		return Ok(metadata_value != IS_DELETE);
	}

	pub async fn set_favorite(&self) -> Result<()> {
		set(&self.path, "user.is_favorite", "true".to_string().as_bytes());

		return Ok(());
	}

	pub async fn unset_favorite(&self) -> Result<()> {
		set(&self.path, "user.is_favorite", "false".to_string().as_bytes());
		return Ok(());
	}

	pub async fn is_favorite(&self) -> Result<bool> {
		const IS_FAVORITE: &str = "false";

		let metadata_vec_value = get(&self.path, "user.is_favorite")
			.unwrap_or(Some(IS_FAVORITE.as_bytes().to_vec()))
			.unwrap_or(IS_FAVORITE.as_bytes().to_vec());
		let metadata_value = str::from_utf8(&metadata_vec_value).unwrap();

		return Ok(metadata_value != IS_FAVORITE);
	}

	pub async fn info(&self) -> Result<FsItem> {
		let metadata = fs::metadata(&self.path).await?;
		let byte = Byte::from_bytes(metadata.len() as u128);
		let size = byte.get_appropriate_unit(true);

		let ext = Path::new(&self.path)
			.extension()
			.unwrap_or(OsStr::new(""))
			.to_str()
			.unwrap_or("");

		let file_type = mime_guess::from_path(&self.path)
			.first()
			.unwrap_or(mime::TEXT_PLAIN)
			.to_string();

		let delete_at = self.get_delete_time().await.unwrap().unwrap_or(0);
		return Ok(FsItem {
			name: self.path.file_name().unwrap().to_str().unwrap().to_string(),
			hash: FsMetadata::calculate_hash(self.base_path.as_ref()),
			size: size.to_string(),
			file_type: ext.to_string(),
			mime_type: file_type,
			is_dir: metadata.is_dir(),
			is_delete: self.is_delete().await.unwrap(),
			is_favorite: self.is_favorite().await?,
			see_time: metadata.atime() as u64,
			delete_at: Some(delete_at as u64),
			buffer: None,
		});
	}

	fn move_dir<'a>(path_from: &'a PathBuf, path_to: &'a PathBuf) -> BoxFuture<'a, ()> {
		async move {
			let mut dir = fs::read_dir(&path_from).await.unwrap();

			while let Some(item) = dir.next_entry().await.unwrap() {
				let item_path_to = Path::new(path_to).join(&item.file_name());
				if item.metadata().await.unwrap().is_dir() {
					fs::create_dir_all(&item_path_to).await.unwrap();
					FsMetadata::move_dir(&item.path(), &item_path_to).await;
					continue;
				}
				fs::copy(&item.path(), &item_path_to).await.unwrap();
			}
		}.boxed()
	}

	pub async fn move_path(&self, path_to: impl AsRef<Path>, is_delete: bool) -> Result<()> {
		if self.is_dir {
			fs::create_dir_all(&path_to).await?;
			FsMetadata::move_dir(&self.path, &path_to.as_ref().to_path_buf()).await;
		} else {
			fs::copy(&self.path, &path_to).await?;
		}

		if is_delete {
			self.remove().await?
		}

		return Ok(());
	}

	pub async fn remove(&self) -> Result<()> {
		let metadata = fs::metadata(&self.path).await?;

		if metadata.is_dir() {
			fs::remove_dir_all(&self.path).await?;
			return Ok(());
		}

		fs::remove_file(&self.path).await?;
		return Ok(());
	}

	pub async fn get_properties(&self) -> Result<PropertiesItem> {
		let path_metadata = fs::metadata(&self.path).await.unwrap();
		let create_at = path_metadata
			.created()
			.unwrap()
			.duration_since(UNIX_EPOCH).unwrap()
			.as_secs();
		let update_at = path_metadata
			.modified()
			.unwrap()
			.duration_since(UNIX_EPOCH)
			.unwrap()
			.as_secs();

		return Ok(PropertiesItem {
			name: self.path.file_name().unwrap().to_str().unwrap().to_string(),
			hash: FsMetadata::calculate_hash(self.base_path.as_ref()),
			is_dir: self.is_dir,
			owner: 0,
			size: path_metadata.size(),
			create_at,
			update_at,
			is_delete: self.is_delete().await.unwrap(),
			is_favorite: self.is_favorite().await.unwrap(),
			see_time: path_metadata.atime() as u64,
			description: "".to_string(),
		})
	}

	pub fn calculate_hash(key: &str) -> String {
		let mut s = DefaultHasher::new();
		key.hash(&mut s);
		let hash = s.finish();
		return format!("{hash:X}");
	}
}
