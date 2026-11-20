use sha2::{Digest, Sha256};

/// SHA-256 of the PDF bytes, lowercase hex.
pub fn content_hash(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::content_hash;

    #[test]
    fn hash_is_stable() {
        let first = content_hash(b"%PDF");
        assert_eq!(first, content_hash(b"%PDF"));
        assert_ne!(first, content_hash(b"%PDF-1"));
        assert_eq!(first.len(), 64);
    }
}
