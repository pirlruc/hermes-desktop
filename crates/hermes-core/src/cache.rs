use rusqlite::{Connection, OptionalExtension};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("cache: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

pub struct ResultCache {
    connection: Connection,
}

impl ResultCache {
    pub fn open(path: &std::path::Path) -> Result<Self, CacheError> {
        let connection = Connection::open(path)?;
        connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS results (
                kind TEXT NOT NULL,
                key TEXT NOT NULL,
                body TEXT NOT NULL,
                PRIMARY KEY (kind, key)
            );",
        )?;
        Ok(Self { connection })
    }

    pub fn put(&self, kind: &str, key: &str, body: &serde_json::Value) -> Result<(), CacheError> {
        self.connection.execute(
            "INSERT INTO results (kind, key, body) VALUES (?1, ?2, ?3)
             ON CONFLICT(kind, key) DO UPDATE SET body = excluded.body",
            (kind, key, body.to_string()),
        )?;
        Ok(())
    }

    pub fn get(&self, kind: &str, key: &str) -> Result<Option<serde_json::Value>, CacheError> {
        let mut statement = self
            .connection
            .prepare("SELECT body FROM results WHERE kind = ?1 AND key = ?2")?;
        let raw = statement
            .query_row((kind, key), |row| row.get::<_, String>(0))
            .optional()?;
        Ok(raw.and_then(|text| serde_json::from_str(&text).ok()))
    }
}

#[cfg(test)]
mod tests {
    use super::ResultCache;

    #[test]
    fn a_stored_row_is_readable_after_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cache.sqlite");
        let cache = ResultCache::open(&path).unwrap();
        cache
            .put("batch-check", "10.1/a", &serde_json::json!({"known": true}))
            .unwrap();
        drop(cache);
        let cache = ResultCache::open(&path).unwrap();
        let body = cache.get("batch-check", "10.1/a").unwrap().unwrap();
        assert_eq!(body["known"], true);
        assert!(cache.get("batch-check", "missing").unwrap().is_none());
    }
}
