use std::collections::hash_map::DefaultHasher;
use std::ffi::OsStr;
use std::hash::{Hash, Hasher};
use std::io::Result;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::str;

use byte_unit::Byte;
use chrono::Utc;
use mime_guess::mime;
use regex::Regex;
use tokio::fs;
use xattr::{get, remove, set};

pub struct Space {
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
	pub see_time: i64,
	pub delete_at: Option<i64>,
	pub buffer: Option<Vec<u8>>,
}

pub struct FsMetadata {
	path: PathBuf,
}

impl FsMetadata {
	pub fn new(path: impl AsRef<Path>) -> Result<FsMetadata> {
		return Ok(FsMetadata {
			path: path.as_ref().to_path_buf()
		});
	}
	
	pub async fn set_delete(&self) -> Result<()> {
		let delete_at = Utc::now().timestamp();
		
		set(&self.path, "user.is_delete", "true".as_bytes());
		set(&self.path, "user.delete_time", delete_at.to_string().as_bytes());
		
		return Ok(());
	}
	
	pub async fn restore_delete(&self) -> Result<()> {
		remove(&self.path, "user.is_delete");
		remove(&self.path, "user.delete_time");
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
	
	pub async fn get_space(&self) -> Result<Space> {
		const AVAILABLE_SPACE: u64 = 0;
		const USAGE_SPACE: u64 = 0;
		
		let available_space = get(&self.path, "user.available_space")
			.unwrap_or(Some(AVAILABLE_SPACE.to_string().as_bytes().to_vec()))
			.unwrap_or(AVAILABLE_SPACE.to_string().as_bytes().to_vec());
		let available_space = str::from_utf8(&available_space).unwrap();
		
		let usage_space = get(&self.path, "user.usage_space")
			.unwrap_or(Some(USAGE_SPACE.to_string().as_bytes().to_vec()))
			.unwrap_or(USAGE_SPACE.to_string().as_bytes().to_vec());
		let usage_space = str::from_utf8(&usage_space).unwrap();
		
		return Ok(Space {
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
	
	pub async fn is_favorite(&self) -> Result<bool> {
		const IS_FAVORITE: &str = "false";
		
		let metadata_vec_value = get(&self.path, "user.is_favorite")
			.unwrap_or(Some(IS_FAVORITE.as_bytes().to_vec()))
			.unwrap_or(IS_FAVORITE.as_bytes().to_vec());
		let metadata_value = str::from_utf8(&metadata_vec_value).unwrap();
		
		return Ok(metadata_value != IS_FAVORITE);
	}
	
	pub async fn info(&self) -> Result<FsItem> {
		let path_regex = Regex::new(r".*([\da-f]{8}-[\da-f]{4}-[\da-f]{4}-[\da-f]{4}-[\da-f]{12})(/files/)").unwrap();
		let base_path = path_regex.replace(self.path.to_str().unwrap(), "");
		
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
		
		let delete_at = self.get_delete_time().await.unwrap();
		return Ok(FsItem {
			name: self.path.file_name().unwrap().to_str().unwrap().to_string(),
			hash: FsMetadata::calculate_hash(base_path.as_ref()),
			size: size.to_string(),
			file_type: ext.to_string(),
			mime_type: file_type,
			is_dir: metadata.is_dir(),
			is_delete: self.is_delete().await.unwrap(),
			is_favorite: self.is_favorite().await?,
			see_time: metadata.atime(),
			delete_at,
			buffer: None,
		});
	}
	
	pub fn calculate_hash(key: &str) -> String {
		let mut s = DefaultHasher::new();
		key.hash(&mut s);
		let hash = s.finish();
		return format!("{hash:X}");
	}
}
