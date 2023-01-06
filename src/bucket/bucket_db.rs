use std::path::Path;

use rusqlite::{Connection, Result};

pub struct BucketDB {}

impl BucketDB {
	pub async fn init(bucket_path: impl AsRef<Path>) -> Result<()> {
		let path = bucket_path.as_ref();
		let mut connection = Connection::open(path.join("user-paths.sqlite"))?;
		
		let transaction = connection.transaction()?;
		
		transaction.execute("CREATE TABLE paths(hash TEXT NOT NULL CONSTRAINT paths_pk PRIMARY KEY,\
		path TEXT NOT NULL, is_dir INTEGER DEFAULT 0 NOT NULL);", ());
		transaction.execute("CREATE TABLE delete_paths(hash TEXT NOT NULL CONSTRAINT delete_paths_pk \
		PRIMARY KEY CONSTRAINT delete_paths_paths_hash_fk REFERENCES paths ON DELETE CASCADE,	delete_time \
		TEXT NOT NULL);", ());
		transaction.execute("CREATE UNIQUE INDEX delete_paths_hash_uindex \
		ON delete_paths (hash);", ());
		transaction.execute("CREATE TABLE favorite_paths(hash TEXT NOT NULL \
		CONSTRAINT favorite_paths_pk PRIMARY KEY CONSTRAINT favorite_paths_paths_hash_fk \
		REFERENCES paths ON DELETE CASCADE);", ());
		transaction.execute("CREATE UNIQUE INDEX favorite_paths_hash_uindex	ON favorite_paths (hash);", ());
		transaction.execute("CREATE TABLE log_paths(hash TEXT NOT NULL CONSTRAINT log_paths_paths_hash_fk \
		REFERENCES paths ON DELETE CASCADE,	author_uuid TEXT NOT NULL, \
		create_at TEXT NOT NULL, comment TEXT NOT NULL);", ());
		transaction.execute("CREATE UNIQUE INDEX paths_path_uindex	ON paths (path);", ());
		transaction.execute("CREATE UNIQUE INDEX paths_paths_uindex	ON paths (hash);", ());
		
		transaction.commit();
		
		return Ok(());
	}
	
	pub async fn open(path: impl AsRef<Path>) -> Result<Connection> {
		return Ok(Connection::open(path)?);
	}
}
