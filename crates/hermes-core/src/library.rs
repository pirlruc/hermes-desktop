use std::path::Path;

use rusqlite::Connection;
use thiserror::Error;

use crate::snapshot::{open_readonly_snapshot, SnapshotError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryWork {
    pub key: String,
    pub title: String,
    pub doi: Option<String>,
    pub pdf_path: Option<String>,
    pub content_hash: Option<String>,
}

#[derive(Debug, Error)]
pub enum LibraryError {
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),
    #[error("library query: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("not a {0} library snapshot")]
    Unrecognized(&'static str),
}

pub struct ZoteroReader;

impl ZoteroReader {
    pub fn read(source: &Path, directory: &Path) -> Result<Vec<LibraryWork>, LibraryError> {
        let (_copy, connection) = open_readonly_snapshot(source, directory)?;
        if !has_table(&connection, "itemAttachments")? {
            return Err(LibraryError::Unrecognized("Zotero"));
        }
        let mut statement = connection.prepare(
            "SELECT parent.key,
                COALESCE((
                    SELECT v.value FROM itemData d
                    JOIN fields f ON f.fieldID = d.fieldID AND f.fieldName = 'title'
                    JOIN itemDataValues v ON v.valueID = d.valueID
                    WHERE d.itemID = parent.itemID
                ), ''),
                (
                    SELECT v.value FROM itemData d
                    JOIN fields f ON f.fieldID = d.fieldID AND f.fieldName = 'DOI'
                    JOIN itemDataValues v ON v.valueID = d.valueID
                    WHERE d.itemID = parent.itemID
                ),
                (
                    SELECT path FROM itemAttachments
                    WHERE parentItemID = parent.itemID
                    LIMIT 1
                )
             FROM items parent
             JOIN itemTypes types ON types.itemTypeID = parent.itemTypeID
             WHERE types.typeName != 'attachment'",
        )?;
        let rows = statement.query_map([], |row| {
            Ok(LibraryWork {
                key: row.get(0)?,
                title: row.get(1)?,
                doi: row.get(2)?,
                pdf_path: row.get(3)?,
                content_hash: None,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(LibraryError::from)
    }
}

pub struct QiqqaReader;

impl QiqqaReader {
    pub fn read(source: &Path, directory: &Path) -> Result<Vec<LibraryWork>, LibraryError> {
        let (_copy, connection) = open_readonly_snapshot(source, directory)?;
        if !has_table(&connection, "documents")? {
            return Err(LibraryError::Unrecognized("Qiqqa"));
        }
        let mut statement =
            connection.prepare("SELECT fingerprint, title, doi, pdf_path FROM documents")?;
        let rows = statement.query_map([], |row| {
            Ok(LibraryWork {
                key: row.get(0)?,
                title: row.get(1)?,
                doi: row.get(2)?,
                pdf_path: row.get(3)?,
                content_hash: None,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(LibraryError::from)
    }
}

fn has_table(connection: &Connection, name: &str) -> Result<bool, rusqlite::Error> {
    let count: i64 = connection.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
        [name],
        |row| row.get(0),
    )?;
    Ok(count == 1)
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::{QiqqaReader, ZoteroReader};

    #[test]
    fn zotero_snapshot_yields_doi_and_pdf_path() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("zotero.sqlite");
        let connection = Connection::open(&source).unwrap();
        connection
            .execute_batch(
                "CREATE TABLE itemTypes (itemTypeID INTEGER, typeName TEXT);
                 CREATE TABLE items (itemID INTEGER, itemTypeID INTEGER, key TEXT);
                 CREATE TABLE fields (fieldID INTEGER, fieldName TEXT);
                 CREATE TABLE itemData (itemID INTEGER, fieldID INTEGER, valueID INTEGER);
                 CREATE TABLE itemDataValues (valueID INTEGER, value TEXT);
                 CREATE TABLE itemAttachments (itemID INTEGER, parentItemID INTEGER, path TEXT);
                 INSERT INTO itemTypes VALUES (1, 'journalArticle'), (2, 'attachment');
                 INSERT INTO items VALUES (1, 1, 'ABCD1234'), (2, 2, 'ATT1');
                 INSERT INTO fields VALUES (1, 'title'), (2, 'DOI');
                 INSERT INTO itemData VALUES (1, 1, 10), (1, 2, 11);
                 INSERT INTO itemDataValues VALUES (10, 'QR codes'), (11, '10.1/qr');
                 INSERT INTO itemAttachments VALUES (2, 1, 'storage/ABCD1234/paper.pdf');",
            )
            .unwrap();
        drop(connection);
        let works = ZoteroReader::read(&source, dir.path()).unwrap();
        assert_eq!(works.len(), 1);
        assert_eq!(works[0].key, "ABCD1234");
        assert_eq!(works[0].doi.as_deref(), Some("10.1/qr"));
        assert_eq!(
            works[0].pdf_path.as_deref(),
            Some("storage/ABCD1234/paper.pdf")
        );
    }

    #[test]
    fn qiqqa_snapshot_yields_fingerprint_and_path() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("qiqqa.sqlite");
        let connection = Connection::open(&source).unwrap();
        connection
            .execute_batch(
                "CREATE TABLE documents (
                    fingerprint TEXT, title TEXT, doi TEXT, pdf_path TEXT
                 );
                 INSERT INTO documents VALUES ('fp1', 'A note', '10.1/q', 'fp1.pdf');",
            )
            .unwrap();
        drop(connection);
        let works = QiqqaReader::read(&source, dir.path()).unwrap();
        assert_eq!(works[0].key, "fp1");
        assert_eq!(works[0].title, "A note");
        assert_eq!(works[0].pdf_path.as_deref(), Some("fp1.pdf"));
    }
}
