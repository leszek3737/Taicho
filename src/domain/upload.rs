use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Returns `true` when `mime` matches the allowed upload types.
///
/// Allowed:
/// - `text/*`
/// - `application/pdf`
/// - `application/json`
/// - `image/*`
/// - `audio/*`
/// - `video/*`
pub fn is_allowed_mime_type(mime: &str) -> bool {
    let mime = mime.trim();
    if mime.is_empty() {
        return false;
    }
    if mime.starts_with("text/")
        || mime.starts_with("image/")
        || mime.starts_with("audio/")
        || mime.starts_with("video/")
    {
        return true;
    }
    matches!(mime, "application/pdf" | "application/json")
}

/// Guess a MIME type from a file extension. Returns `None` when unknown.
pub fn mime_from_extension(path: &Path) -> Option<String> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    let mime = match ext.as_str() {
        // text/*
        "txt" | "csv" | "tsv" | "log" | "md" | "markdown" | "html" | "htm" | "xml" | "yaml"
        | "yml" | "toml" | "ini" | "cfg" | "conf" | "sh" | "bash" | "zsh" | "fish" | "ps1"
        | "bat" | "cmd" | "py" | "rb" | "js" | "jsx" | "tsx" | "rs" | "go" | "java" | "c" | "h"
        | "cpp" | "hpp" | "cs" | "swift" | "kt" | "scala" | "clj" | "lua" | "pl" | "pm" | "r"
        | "sql" | "css" | "scss" | "less" | "graphql" | "gql" | "proto" | "tf" | "hcl"
        | "dockerfile" | "makefile" | "cmake" | "gradle" | "properties" | "env" | "gitignore"
        | "editorconfig" | "license" | "readme" => "text/plain",
        "ts" => "text/plain", // TypeScript (not MPEG-TS)
        "json" | "jsonl" => "application/json",
        "pdf" => "application/pdf",
        // image/*
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" | "svgz" => "image/svg+xml",
        "bmp" => "image/bmp",
        "ico" => "image/x-icon",
        "tiff" | "tif" => "image/tiff",
        "avif" => "image/avif",
        "heic" | "heif" => "image/heif",
        // audio/*
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "flac" => "audio/flac",
        "aac" => "audio/aac",
        "m4a" => "audio/mp4",
        "opus" => "audio/opus",
        "wma" => "audio/x-ms-wma",
        "mid" | "midi" => "audio/midi",
        // video/*
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "mkv" => "video/x-matroska",
        "avi" => "video/x-msvideo",
        "mov" => "video/quicktime",
        "wmv" => "video/x-ms-wmv",
        "flv" => "video/x-flv",
        "ogv" => "video/ogg",
        "m2ts" | "mts" => "video/mp2t",
        _ => return None,
    };
    Some(mime.to_owned())
}

/// Human-readable list of accepted file types for UI display.
pub const ACCEPTED_TYPES_DISPLAY: &str =
    "text/*, application/pdf, application/json, image/*, audio/*, video/*";

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

    #[test]
    fn allowed_mime_types() {
        assert!(is_allowed_mime_type("text/plain"));
        assert!(is_allowed_mime_type("text/html"));
        assert!(is_allowed_mime_type("text/csv"));
        assert!(is_allowed_mime_type("application/pdf"));
        assert!(is_allowed_mime_type("application/json"));
        assert!(is_allowed_mime_type("image/png"));
        assert!(is_allowed_mime_type("image/jpeg"));
        assert!(is_allowed_mime_type("audio/mpeg"));
        assert!(is_allowed_mime_type("audio/wav"));
        assert!(is_allowed_mime_type("video/mp4"));
        assert!(is_allowed_mime_type("video/webm"));
    }

    #[test]
    fn disallowed_mime_types() {
        assert!(!is_allowed_mime_type("application/octet-stream"));
        assert!(!is_allowed_mime_type("application/zip"));
        assert!(!is_allowed_mime_type("application/javascript"));
        assert!(!is_allowed_mime_type("font/woff2"));
        assert!(!is_allowed_mime_type("model/gltf-binary"));
        assert!(!is_allowed_mime_type(""));
        assert!(!is_allowed_mime_type("   "));
    }

    #[test]
    fn mime_from_extension_known() {
        assert_eq!(
            mime_from_extension(Path::new("notes.txt")),
            Some("text/plain".to_owned())
        );
        assert_eq!(
            mime_from_extension(Path::new("data.json")),
            Some("application/json".to_owned())
        );
        assert_eq!(
            mime_from_extension(Path::new("doc.pdf")),
            Some("application/pdf".to_owned())
        );
        assert_eq!(
            mime_from_extension(Path::new("photo.jpg")),
            Some("image/jpeg".to_owned())
        );
        assert_eq!(
            mime_from_extension(Path::new("song.mp3")),
            Some("audio/mpeg".to_owned())
        );
        assert_eq!(
            mime_from_extension(Path::new("clip.mp4")),
            Some("video/mp4".to_owned())
        );
    }

    #[test]
    fn mime_from_extension_unknown() {
        assert_eq!(mime_from_extension(Path::new("file.unknownext")), None);
        assert_eq!(mime_from_extension(Path::new("noext")), None);
    }

    #[test]
    fn mime_from_extension_case_insensitive() {
        assert_eq!(
            mime_from_extension(Path::new("FILE.PDF")),
            Some("application/pdf".to_owned())
        );
        assert_eq!(
            mime_from_extension(Path::new("image.PNG")),
            Some("image/png".to_owned())
        );
    }
}
