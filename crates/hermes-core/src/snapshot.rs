use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::Connection;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SnapshotError {
    #[error("copy snapshot: {0}")]
    Copy(#[from] std::io::Error),
    #[error("open snapshot: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

/// Copy a live database, then open the copy read-only with `immutable=1`.
pub fn open_readonly_snapshot(
    source: &Path,
    directory: &Path,
) -> Result<(PathBuf, Connection), SnapshotError> {
    let destination = directory.join("library.sqlite");
    fs::copy(source, &destination)?;
    let uri = format!(
        "file:{}?mode=ro&immutable=1",
        destination.to_string_lossy().replace('\\', "/")
    );
    let connection = Connection::open(uri)?;
    Ok((destination, connection))
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::open_readonly_snapshot;

    #[test]
    fn snapshot_is_readable_and_not_the_source_path() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("live.sqlite");
        let connection = Connection::open(&source).unwrap();
        connection
            .execute_batch("CREATE TABLE items (id INTEGER); INSERT INTO items VALUES (1);")
            .unwrap();
        drop(connection);
        let snap_dir = dir.path().join("snap");
        std::fs::create_dir(&snap_dir).unwrap();
        let (copied, snapshot) = open_readonly_snapshot(&source, &snap_dir).unwrap();
        assert_ne!(copied, source);
        let count: i64 = snapshot
            .query_row("SELECT COUNT(*) FROM items", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
        let write = snapshot.execute("DELETE FROM items", []);
        assert!(write.is_err());
    }
}
