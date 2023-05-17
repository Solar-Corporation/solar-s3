use std::path::Path;
use std::str::from_utf8;

use rusqlite::{Connection, Result, Transaction};
use tokio::fs;

use crate::bucket::fs_metadata::FsMetadata;

#[derive(Debug)]
pub struct KeyPath {
	pub key: String,
	pub path: String,
	pub is_dir: bool,
}

pub struct BucketDB;

impl BucketDB {
	pub async fn init(bucket_path: impl AsRef<Path>) -> Result<()> {
		let path = bucket_path.as_ref();
		let mut connection = Connection::open(path.join("user-paths.sqlite"))?;

		let transaction = connection.transaction()?;

		let sql_scripts = fs::read("./src/bucket/sql/create_tables.sql").await.unwrap();

		let create_tables = from_utf8(&sql_scripts).unwrap();
		for script in create_tables.split(";") {
			transaction.execute(&script, ()).unwrap();
		}

		transaction.commit();
		connection.execute("PRAGMA foreign_keys = ON", ()).unwrap();

		return Ok(());
	}

	pub async fn open(path: impl AsRef<Path>) -> Result<Connection> {
		let path = Path::new(path.as_ref()).join("user-paths.sqlite");
		return Ok(Connection::open(path)?);
	}

	pub async fn add_key(key: &KeyPath, transaction: &Transaction<'_>) -> Result<()> {
		transaction.execute("INSERT INTO paths (hash, path, is_dir) VALUES (?1, ?2, ?3);", (&key.key, &key.path, key.is_dir as i8))
			.unwrap();
		return Ok(());
	}

	pub async fn get_path(key: &str, transaction: &Transaction<'_>) -> Result<String> {
		transaction.query_row("SELECT path FROM paths WHERE hash = ?1",
							  [key], |row| {
				row.get(0)
			})
	}

	pub async fn update_paths(old_path: &str, new_path: &str, transaction: &Transaction<'_>) -> Result<Vec<String>> {
		let mut prepare_query = transaction.prepare("SELECT * FROM paths WHERE path LIKE ?1 || '%'").unwrap();
		let key_paths = prepare_query.query_map([old_path], |row| {
			Ok(KeyPath {
				key: row.get(0).unwrap(),
				path: row.get(1).unwrap(),
				is_dir: row.get::<_, u8>(2).unwrap() != 0,
			})
		}).unwrap();
		let mut vec_hashes: Vec<String> = Vec::new();

		for key_path in key_paths {
			let key_path = key_path.unwrap();
			let updated_path = key_path.path.replacen(old_path, new_path, 1);

			let updated_hash = FsMetadata::calculate_hash(updated_path.as_str());
			vec_hashes.push(updated_hash.clone());
			transaction.execute("UPDATE paths SET hash = ?1, path = ?2 WHERE hash = ?3", [updated_hash, updated_path, key_path.key]);
		}

		return Ok(vec_hashes);
	}

	pub async fn copy_paths(from_path: &str, copy_path: &str, transaction: &Transaction<'_>) -> Result<Vec<String>> {
		let mut prepare_query = transaction.prepare("SELECT * FROM paths WHERE path LIKE ?1 || '%'").unwrap();
		let key_paths = prepare_query.query_map([from_path], |row| {
			Ok(KeyPath {
				key: row.get(0).unwrap(),
				path: row.get(1).unwrap(),
				is_dir: row.get::<_, u8>(2).unwrap() != 0,
			})
		}).unwrap();
		let mut vec_hashes: Vec<String> = Vec::new();

		for key_path in key_paths {
			let key_path = key_path.unwrap();
			let updated_path = key_path.path.replacen(from_path, copy_path, 1);
			let updated_hash = FsMetadata::calculate_hash(updated_path.as_str());
			vec_hashes.push(updated_hash.clone());
			transaction.execute("INSERT INTO paths (hash, path, is_dir) VALUES (?1, ?2, ?3);", [updated_hash, updated_path, key_path.key]).unwrap();
		}

		return Ok(vec_hashes);
	}

	pub async fn set_favorite(key: &str, transaction: &Transaction<'_>) -> Result<()> {
		transaction.execute("INSERT INTO favorite_paths (hash) VALUES (?1);", [key]).unwrap();
		return Ok(());
	}

	pub async fn unset_favorite(key: &str, transaction: &Transaction<'_>) -> Result<()> {
		transaction.execute("DELETE from favorite_paths WHERE hash = ?1", [key]).unwrap();
		return Ok(());
	}

	pub async fn get_favorites(transaction: &Transaction<'_>) -> Result<Vec<String>> {
		let mut prepare_query = transaction.prepare("SELECT path FROM favorite_paths INNER JOIN paths p ON p.hash = favorite_paths.hash").unwrap();
		let key_paths = prepare_query.query_map([], |row| {
			Ok(row.get::<_, String>(0))
		}).unwrap();
		let mut vec_hashes: Vec<String> = Vec::new();

		for key_path in key_paths {
			let key_path = key_path.unwrap().unwrap();
			vec_hashes.push(key_path.clone());
		}

		return Ok(vec_hashes);
	}

	pub async fn set_delete(key: &str, date: i64, transaction: &Transaction<'_>) -> Result<()> {
		transaction.execute("INSERT INTO delete_paths (hash, delete_time) VALUES (?1, ?2);", (key, date)).unwrap();
		return Ok(());
	}

	pub async fn restore_delete(key: &str, transaction: &Transaction<'_>) -> Result<()> {
		transaction.execute("DELETE FROM delete_paths WHERE hash = ?1", [key]).unwrap();
		return Ok(());
	}

	pub async fn get_deletes(transaction: &Transaction<'_>) -> Result<Vec<String>> {
		let mut prepare_query = transaction.prepare("SELECT path FROM delete_paths INNER JOIN paths p ON p.hash = delete_paths.hash").unwrap();
		let key_paths = prepare_query.query_map([], |row| {
			Ok(row.get::<_, String>(0))
		}).unwrap();
		let mut vec_hashes: Vec<String> = Vec::new();

		for key_path in key_paths {
			let key_path = key_path.unwrap().unwrap();
			vec_hashes.push(key_path.clone());
		}

		return Ok(vec_hashes);
	}

	pub async fn clear_trash(transaction: &Transaction<'_>) -> Result<Vec<String>> {
		let mut prepare_query = transaction.prepare("SELECT hash, path, is_dir from paths p join (SELECT hash AS delete_hash FROM delete_paths ) AS dp on p.hash = dp.delete_hash").unwrap();
		let key_paths = prepare_query.query_map([], |row| {
			Ok(KeyPath {
				key: row.get(0).unwrap(),
				path: row.get(1).unwrap(),
				is_dir: row.get::<_, u8>(2).unwrap() != 0,
			})
		}).unwrap();
		let mut vec_hashes: Vec<String> = Vec::new();

		for key_path in key_paths {
			let key_path = key_path.unwrap();
			vec_hashes.push(key_path.path);
			transaction.execute("DELETE FROM paths WHERE hash = ?1", [key_path.key]).unwrap();
		}

		return Ok(vec_hashes);
	}

	pub async fn remove_trash(key: &str, transaction: &Transaction<'_>) -> Result<()> {
		transaction.execute("DELETE FROM paths WHERE hash = ?1", [key]).unwrap();
		return Ok(());
	}

	pub async fn remove_date_trash(date: i64, transaction: &Transaction<'_>) -> Result<Vec<String>> {
		let mut prepare_query = transaction.prepare("SELECT path, delete_time FROM delete_paths INNER JOIN paths p ON p.hash = delete_paths.hash WHERE delete_time < ?1").unwrap();
		let key_paths = prepare_query.query_map([], |row| {
			Ok(KeyPath {
				key: row.get(0).unwrap(),
				path: row.get(1).unwrap(),
				is_dir: row.get::<_, u8>(2).unwrap() != 0,
			})
		}).unwrap();
		let mut vec_hashes: Vec<String> = Vec::new();

		for key_path in key_paths {
			let key_path = key_path.unwrap();
			vec_hashes.push(key_path.path);
			transaction.execute("DELETE FROM delete_paths WHERE hash = ?1", [key_path.key]).unwrap();
		}

		return Ok(vec_hashes);
	}
}
