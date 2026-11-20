use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::hash::content_hash;
use crate::library::LibraryWork;

/// Delphi batch-check accepts at most this many identities per request.
pub const BATCH_LIMIT: usize = 500;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Identity {
    pub kind: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchRow {
    pub identity: Identity,
    pub known: bool,
    pub work_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncAction {
    Known { key: String, work_id: String },
    Queued { key: String },
    Unresolved { key: String },
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("batch-check rejected a chunk larger than {BATCH_LIMIT}")]
    ChunkTooLarge,
    #[error("delphi request failed: {0}")]
    Transport(String),
}

#[derive(Debug, Clone)]
pub struct DelphiClient {
    base_url: String,
    api_key: String,
}

impl DelphiClient {
    pub fn new(base_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        let base_url = base_url.into().trim_end_matches('/').to_string();
        Self {
            base_url,
            api_key: api_key.into(),
        }
    }

    pub fn batch_check(&self, identities: &[Identity]) -> Result<Vec<BatchRow>, ClientError> {
        if identities.is_empty() {
            return Ok(Vec::new());
        }
        if identities.len() > BATCH_LIMIT {
            return Err(ClientError::ChunkTooLarge);
        }
        let response = ureq::post(&format!("{}/v1/works:batch-check", self.base_url))
            .set("X-API-Key", &self.api_key)
            .send_json(serde_json::json!({ "identities": identities }))
            .map_err(|err| ClientError::Transport(err.to_string()))?;
        let body: ResponseBody = response
            .into_json()
            .map_err(|err| ClientError::Transport(err.to_string()))?;
        Ok(body.rows)
    }
}

#[derive(Deserialize)]
struct ResponseBody {
    rows: Vec<BatchRow>,
}

pub fn identities_for(work: &LibraryWork, pdf: Option<&[u8]>) -> Vec<Identity> {
    let mut identities = Vec::new();
    if let Some(doi) = work.doi.as_deref().filter(|value| !value.is_empty()) {
        identities.push(Identity {
            kind: "doi".to_string(),
            value: doi.to_string(),
        });
    }
    if let Some(bytes) = pdf {
        identities.push(Identity {
            kind: "content_hash".to_string(),
            value: content_hash(bytes),
        });
    }
    identities
}

pub fn chunk<T>(items: &[T], size: usize) -> impl Iterator<Item = &[T]> {
    let size = size.max(1);
    items.chunks(size)
}

pub fn plan(works: &[LibraryWork], rows: &[BatchRow]) -> Vec<SyncAction> {
    works
        .iter()
        .map(|work| {
            let known = rows.iter().find(|row| {
                work_matches(work, &row.identity) && row.known && row.work_id.is_some()
            });
            if let Some(row) = known {
                SyncAction::Known {
                    key: work.key.clone(),
                    work_id: row.work_id.clone().unwrap_or_default(),
                }
            } else if work.pdf_path.is_some() {
                SyncAction::Queued {
                    key: work.key.clone(),
                }
            } else {
                SyncAction::Unresolved {
                    key: work.key.clone(),
                }
            }
        })
        .collect()
}

fn work_matches(work: &LibraryWork, identity: &Identity) -> bool {
    match identity.kind.as_str() {
        "doi" => work.doi.as_deref() == Some(identity.value.as_str()),
        "content_hash" => work.content_hash.as_deref() == Some(identity.value.as_str()),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{chunk, identities_for, plan, BatchRow, Identity, SyncAction, BATCH_LIMIT};
    use crate::library::LibraryWork;

    fn work(key: &str, doi: Option<&str>, pdf: bool) -> LibraryWork {
        LibraryWork {
            key: key.to_string(),
            title: key.to_string(),
            doi: doi.map(str::to_string),
            pdf_path: pdf.then(|| format!("{key}.pdf")),
            content_hash: pdf.then(|| "abc".to_string()),
        }
    }

    #[test]
    fn chunks_stop_at_the_batch_limit() {
        let items: Vec<u16> = (0..501).collect();
        let parts: Vec<_> = chunk(&items, BATCH_LIMIT).collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].len(), 500);
        assert_eq!(parts[1].len(), 1);
    }

    #[test]
    fn identities_include_doi_and_hash() {
        let work = work("a", Some("10.1/x"), false);
        let ids = identities_for(&work, Some(b"%PDF"));
        assert_eq!(ids[0].kind, "doi");
        assert_eq!(ids[1].kind, "content_hash");
        assert_eq!(ids[1].value.len(), 64);
    }

    #[test]
    fn plan_pulls_metadata_on_hit_and_queues_a_miss() {
        let works = vec![
            work("known", Some("10.1/a"), true),
            work("miss", Some("10.1/b"), true),
        ];
        let rows = vec![
            BatchRow {
                identity: Identity {
                    kind: "doi".to_string(),
                    value: "10.1/a".to_string(),
                },
                known: true,
                work_id: Some("w-1".to_string()),
            },
            BatchRow {
                identity: Identity {
                    kind: "doi".to_string(),
                    value: "10.1/b".to_string(),
                },
                known: false,
                work_id: None,
            },
        ];
        assert_eq!(
            plan(&works, &rows),
            vec![
                SyncAction::Known {
                    key: "known".to_string(),
                    work_id: "w-1".to_string(),
                },
                SyncAction::Queued {
                    key: "miss".to_string(),
                },
            ]
        );
    }

    #[test]
    fn a_miss_without_a_pdf_stays_unresolved() {
        let works = vec![work("note", Some("10.1/c"), false)];
        let actions = plan(&works, &[]);
        assert_eq!(
            actions,
            vec![SyncAction::Unresolved {
                key: "note".to_string()
            }]
        );
    }
}
