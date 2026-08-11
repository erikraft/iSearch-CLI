//! Central URI parsing, routing, and media classification for iSearch CLI™.

use base64::Engine;
use std::path::PathBuf;

const DATA_LIMIT: usize = 2 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UriScheme {
    Http,
    Https,
    Ws,
    Wss,
    Ftp,
    Ftps,
    Sftp,
    Smtp,
    Smtps,
    Data,
    Blob,
    File,
    LocalPath,
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedUri {
    pub original: String,
    pub scheme: UriScheme,
    pub normalized: String,
    pub local_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediaKind {
    Image,
    Video,
    Audio,
    Document,
    Archive,
    Directory,
    File,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataUri {
    pub mime_type: String,
    pub is_base64: bool,
    pub bytes: Vec<u8>,
}

pub fn parse(input: &str) -> ParsedUri {
    let trimmed = input.trim();
    if is_windows_drive_path(trimmed) {
        return local(trimmed);
    }
    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("file:") {
        return file_uri(trimmed);
    }
    for (prefix, scheme) in [
        ("https://", UriScheme::Https),
        ("http://", UriScheme::Http),
        ("wss://", UriScheme::Wss),
        ("ws://", UriScheme::Ws),
        ("ftps://", UriScheme::Ftps),
        ("ftp://", UriScheme::Ftp),
        ("sftp://", UriScheme::Sftp),
        ("smtps://", UriScheme::Smtps),
        ("smtp://", UriScheme::Smtp),
    ] {
        if lower.starts_with(prefix) {
            return ParsedUri {
                original: trimmed.into(),
                scheme,
                normalized: trimmed.into(),
                local_path: None,
            };
        }
    }
    if lower.starts_with("data:") || lower.starts_with("data://") {
        return ParsedUri {
            original: trimmed.into(),
            scheme: UriScheme::Data,
            normalized: trimmed.into(),
            local_path: None,
        };
    }
    if lower.starts_with("blob:") || lower.starts_with("blob://") {
        return ParsedUri {
            original: trimmed.into(),
            scheme: UriScheme::Blob,
            normalized: trimmed.into(),
            local_path: None,
        };
    }
    if std::path::Path::new(trimmed).exists()
        || trimmed.contains(std::path::MAIN_SEPARATOR)
        || trimmed.contains('\\')
    {
        local(trimmed)
    } else if let Some(idx) = trimmed.find(':') {
        ParsedUri {
            original: trimmed.into(),
            scheme: UriScheme::Unknown(trimmed[..idx].into()),
            normalized: trimmed.into(),
            local_path: None,
        }
    } else {
        ParsedUri {
            original: trimmed.into(),
            scheme: UriScheme::Https,
            normalized: format!("https://{}", trimmed),
            local_path: None,
        }
    }
}

fn local(s: &str) -> ParsedUri {
    ParsedUri {
        original: s.into(),
        scheme: UriScheme::LocalPath,
        normalized: s.into(),
        local_path: Some(PathBuf::from(s)),
    }
}

fn file_uri(s: &str) -> ParsedUri {
    let rest = &s[5..];
    let path = if is_windows_drive_path(rest) {
        rest.to_string()
    } else if let Some(stripped) = rest.strip_prefix("///") {
        decode_file_path(stripped)
    } else if let Some(stripped) = rest.strip_prefix("//") {
        decode_file_path(stripped)
    } else {
        decode_file_path(rest)
    };
    ParsedUri {
        original: s.into(),
        scheme: UriScheme::File,
        normalized: format!("file:{}", path),
        local_path: Some(PathBuf::from(path)),
    }
}

fn decode_file_path(s: &str) -> String {
    percent_encoding::percent_decode_str(s)
        .decode_utf8_lossy()
        .replace('/', std::path::MAIN_SEPARATOR_STR)
}

pub fn is_windows_drive_path(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() >= 3
        && b[1] == b':'
        && (b[0] as char).is_ascii_alphabetic()
        && (b[2] == b'\\' || b[2] == b'/')
}

pub fn classify_path(path: &std::path::Path) -> MediaKind {
    if path.is_dir() {
        return MediaKind::Directory;
    }
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match ext.as_str() {
        "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp" | "tiff" | "tif" | "svg" | "ico"
        | "avif" => MediaKind::Image,
        "mp4" | "mov" | "mkv" | "webm" | "avi" | "m4v" | "flv" | "wmv" | "3gp" | "ts" | "mpeg"
        | "mpg" => MediaKind::Video,
        "mp3" | "ogg" | "oga" | "wav" | "flac" | "m4a" | "aac" | "opus" | "wma" | "aiff" => {
            MediaKind::Audio
        }
        "zip" | "tar" | "gz" | "tgz" | "bz2" | "xz" | "7z" | "rar" => MediaKind::Archive,
        "pdf" | "md" | "txt" | "doc" | "docx" | "rtf" | "odt" => MediaKind::Document,
        _ if path.exists() => MediaKind::File,
        _ => MediaKind::Unknown,
    }
}

pub fn parse_data_uri(input: &str) -> Result<DataUri, String> {
    let payload = input
        .strip_prefix("data:")
        .or_else(|| input.strip_prefix("data://"))
        .ok_or("not a data URI")?;
    let (meta, data) = payload.split_once(',').ok_or("data URI missing comma")?;
    if data.len() > DATA_LIMIT {
        return Err("data URI exceeds size limit".into());
    }
    let mut mime_type = "text/plain".to_string();
    let mut is_base64 = false;
    for (i, p) in meta.split(';').enumerate() {
        if i == 0 && !p.is_empty() {
            mime_type = p.to_string();
        }
        if p.eq_ignore_ascii_case("base64") {
            is_base64 = true;
        }
    }
    let bytes = if is_base64 {
        base64::engine::general_purpose::STANDARD
            .decode(data)
            .map_err(|e| e.to_string())?
    } else {
        percent_encoding::percent_decode_str(data).collect()
    };
    Ok(DataUri {
        mime_type,
        is_base64,
        bytes,
    })
}

pub fn autocomplete_prefixes() -> &'static [&'static str] {
    &[
        "http://", "https://", "ws://", "wss://", "ftp://", "ftps://", "sftp://", "smtp://",
        "smtps://", "data:", "blob:", "file:", "C:\\\\", "D:\\\\", "E:\\\\",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn required_uris() {
        for (s, sch) in [
            ("http://example.com", UriScheme::Http),
            ("https://example.com", UriScheme::Https),
            ("ws://example.com/socket", UriScheme::Ws),
            ("wss://example.com/socket", UriScheme::Wss),
            ("ftp://example.com/file.zip", UriScheme::Ftp),
            ("ftps://example.com/file.zip", UriScheme::Ftps),
            ("sftp://server/path/file.zip", UriScheme::Sftp),
            ("smtp://mail.example.com", UriScheme::Smtp),
            ("data:text/plain,Hello", UriScheme::Data),
            ("data:text/plain;base64,SGVsbG8=", UriScheme::Data),
            ("blob:https://example.com/uuid", UriScheme::Blob),
            ("blob://example/resource", UriScheme::Blob),
            ("file:C:\\Users\\Vera\\Downloads\\me.png", UriScheme::File),
            ("file:///C:/Users/Vera/Downloads/me.png", UriScheme::File),
            ("C:\\Users\\Vera\\Downloads\\me.png", UriScheme::LocalPath),
        ] {
            assert_eq!(parse(s).scheme, sch, "{s}");
        }
    }
    #[test]
    fn media_exts() {
        for e in ["png", "jpg", "webp"] {
            assert_eq!(
                classify_path(&PathBuf::from(format!("a.{e}"))),
                MediaKind::Image
            )
        }
        for e in ["mp4", "mov", "mkv", "webm"] {
            assert_eq!(
                classify_path(&PathBuf::from(format!("a.{e}"))),
                MediaKind::Video
            )
        }
        for e in ["mp3", "ogg", "wav", "flac", "m4a", "opus"] {
            assert_eq!(
                classify_path(&PathBuf::from(format!("a.{e}"))),
                MediaKind::Audio
            )
        }
    }
    #[test]
    fn data_decodes() {
        assert_eq!(
            parse_data_uri("data:text/plain,Hello").unwrap().bytes,
            b"Hello"
        );
        assert_eq!(
            parse_data_uri("data:text/plain;base64,SGVsbG8=")
                .unwrap()
                .bytes,
            b"Hello"
        );
    }
}
