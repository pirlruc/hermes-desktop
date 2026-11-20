//! Local library sync against a pinned Delphi Engine release.

mod cache;
mod consent;
mod hash;
mod library;
mod snapshot;
mod sync;

pub use cache::ResultCache;
pub use consent::{upload_allowed, ConsentRequired};
pub use hash::content_hash;
pub use library::{LibraryWork, QiqqaReader, ZoteroReader};
pub use snapshot::open_readonly_snapshot;
pub use sync::{
    chunk, identities_for, plan, BatchRow, DelphiClient, Identity, SyncAction, BATCH_LIMIT,
};
