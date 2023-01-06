use std::io::Result;
use std::path::Path;
use std::str;

use chrono::Utc;
use xattr::{get, remove, set};

pub struct Space {
	pub available_space: u64,
	pub usage_space: u64,
}

pub struct FsMetadata {}

impl FsMetadata {
	pub async fn set_delete(path: impl AsRef<Path>) -> Result<()> {
		let delete_at = Utc::now().timestamp();
		
		set(&path, "user.is_delete", "true".as_bytes());
		set(&path, "user.delete_time", delete_at.to_string().as_bytes());
		
		return Ok(());
	}
	
	pub async fn restore_delete(path: impl AsRef<Path>) -> Result<()> {
		remove(&path, "user.is_delete");
		remove(&path, "user.delete_time");
		return Ok(());
	}
	
	pub async fn set_available_space(path: impl AsRef<Path>, available_space: u64) -> Result<()> {
		set(&path, "user.available_space", available_space.to_string().as_bytes());
		return Ok(());
	}
	
	pub async fn get_space(path: impl AsRef<Path>) -> Result<Space> {
		const AVAILABLE_SPACE: u64 = 0;
		const USAGE_SPACE: u64 = 0;
		
		let available_space = get(&path, "user.available_space")
			.unwrap_or(Some(AVAILABLE_SPACE.to_string().as_bytes().to_vec()))
			.unwrap_or(AVAILABLE_SPACE.to_string().as_bytes().to_vec());
		let available_space = str::from_utf8(&available_space).unwrap();
		
		let usage_space = get(&path, "user.usage_space")
			.unwrap_or(Some(USAGE_SPACE.to_string().as_bytes().to_vec()))
			.unwrap_or(USAGE_SPACE.to_string().as_bytes().to_vec());
		let usage_space = str::from_utf8(&usage_space).unwrap();
		
		return Ok(Space {
			available_space: available_space.parse::<u64>().unwrap(),
			usage_space: usage_space.parse::<u64>().unwrap(),
		});
	}
	
	pub async fn is_delete(path: impl AsRef<Path>) -> Result<bool> {
		const IS_DELETE: &str = "false";
		
		let metadata_vec_value = get(&path, "user.is_delete")
			.unwrap_or(Some(IS_DELETE.as_bytes().to_vec()))
			.unwrap_or(IS_DELETE.as_bytes().to_vec());
		let metadata_value = str::from_utf8(&metadata_vec_value).unwrap();
		
		return Ok(metadata_value != IS_DELETE);
	}
	
	pub async fn is_favorite(path: impl AsRef<Path>) -> Result<bool> {
		const IS_FAVORITE: &str = "false";
		
		let metadata_vec_value = get(&path, "user.is_favorite")
			.unwrap_or(Some(IS_FAVORITE.as_bytes().to_vec()))
			.unwrap_or(IS_FAVORITE.as_bytes().to_vec());
		let metadata_value = str::from_utf8(&metadata_vec_value).unwrap();
		
		return Ok(metadata_value != IS_FAVORITE);
	}
}
