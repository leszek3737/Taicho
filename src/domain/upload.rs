use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileSource {
    Path(PathBuf),
    Bytes {
        name: String,
        mime: String,
        data: Vec<u8>,
    },
}

impl FileSource {
    pub fn name(&self) -> &str {
        match self {
            Self::Path(p) => {
                let Some(s) = p.file_name().and_then(|n| n.to_str()) else {
                    return "";
                };
                s
            }
            Self::Bytes { name, .. } => name,
        }
    }

    /// Blocking I/O — call from actor only.
    pub fn size_hint(&self) -> Option<u64> {
        match self {
            Self::Path(p) => p.metadata().ok().map(|m| m.len()),
            Self::Bytes { data, .. } => Some(data.len() as u64),
        }
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::print_stderr
)]
mod tests {
    use super::*;

    #[test]
    fn file_source_bytes_name_and_size() {
        let src = FileSource::Bytes {
            name: "notes.txt".to_string(),
            mime: "text/plain".to_string(),
            data: vec![1, 2, 3, 4],
        };
        assert_eq!(src.name(), "notes.txt");
        assert_eq!(src.size_hint(), Some(4));
    }

    #[test]
    fn file_source_path_size_hint_from_metadata() {
        let src = FileSource::Path(PathBuf::from(file!()));
        let hint = src.size_hint();
        assert!(hint.is_some());
        assert!(hint.unwrap() > 0);
        let name = src.name();
        assert_eq!(name, "upload.rs");
    }
}
