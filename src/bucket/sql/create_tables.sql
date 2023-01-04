CREATE TABLE paths
(
	hash   TEXT              NOT NULL
		CONSTRAINT paths_pk
			PRIMARY KEY,
	path   TEXT              NOT NULL,
	is_dir INTEGER DEFAULT 0 NOT NULL
);

CREATE TABLE delete_paths
(
	hash        TEXT NOT NULL
		CONSTRAINT delete_paths_pk
			PRIMARY KEY
		CONSTRAINT delete_paths_paths_hash_fk
			REFERENCES paths
			ON DELETE CASCADE,
	delete_time TEXT NOT NULL
);

CREATE UNIQUE INDEX delete_paths_hash_uindex
	ON delete_paths (hash);

CREATE TABLE favorite_paths
(
	hash TEXT NOT NULL
		CONSTRAINT favorite_paths_pk
			PRIMARY KEY
		CONSTRAINT favorite_paths_paths_hash_fk
			REFERENCES paths
			ON DELETE CASCADE
);

CREATE UNIQUE INDEX favorite_paths_hash_uindex
	ON favorite_paths (hash);

CREATE TABLE log_paths
(
	hash        TEXT NOT NULL
		CONSTRAINT log_paths_paths_hash_fk
			REFERENCES paths
			ON DELETE CASCADE,
	author_uuid TEXT NOT NULL,
	create_at   TEXT NOT NULL,
	comment     TEXT NOT NULL
);

CREATE UNIQUE INDEX paths_path_uindex
	ON paths (path);

CREATE UNIQUE INDEX paths_paths_uindex
	ON paths (hash);

