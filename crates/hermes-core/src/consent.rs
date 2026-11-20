use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
#[error("upload refused: no consent for {key}")]
pub struct ConsentRequired {
    pub key: String,
}

/// Upload is allowed only after an explicit grant for that library key.
pub fn upload_allowed(granted: bool, key: &str) -> Result<(), ConsentRequired> {
    if granted {
        Ok(())
    } else {
        Err(ConsentRequired {
            key: key.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::upload_allowed;

    #[test]
    fn missing_consent_blocks_upload() {
        let err = upload_allowed(false, "item").unwrap_err();
        assert_eq!(err.key, "item");
        assert!(upload_allowed(true, "item").is_ok());
    }
}
