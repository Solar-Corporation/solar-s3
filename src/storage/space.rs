use std::path::PathBuf;

use futures::future::{BoxFuture, FutureExt};
#[cfg(test)]
use mocktopus::macros::*;
use tokio::fs;

pub struct Space;

#[cfg_attr(test, mockable)]
impl Space {
	pub fn get_disc() -> u64 {
		return 1099511627776;
	}
	
	pub fn dir_size(dir_path: &PathBuf) -> BoxFuture<u64> {
		async move {
			let mut total_size = 0;
			let mut dir = fs::read_dir(dir_path).await.unwrap();
			while let Some(item) = dir.next_entry().await.unwrap() {
				if item.metadata().await.unwrap().is_dir() {
					total_size += Space::dir_size(&item.path()).await;
					continue;
				}
				total_size += item.metadata().await.unwrap().len();
			}
			return total_size;
		}.boxed()
	}
}
