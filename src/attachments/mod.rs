//! Attachment handling for images and files

mod image;
mod file;

pub use image::process_image;
pub use file::read_file_attachment;

use std::path::Path;
use anyhow::Result;

/// Supported attachment types
#[derive(Debug, Clone)]
pub enum AttachmentType {
    Image,
    Text,
    Binary,
}

/// Detect attachment type from path
pub fn detect_type(path: &Path) -> AttachmentType {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        // Images
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" => AttachmentType::Image,
        // Text files
        "txt" | "md" | "rs" | "py" | "js" | "ts" | "json" | "toml" | "yaml" | "yml"
        | "html" | "css" | "xml" | "sh" | "bash" | "c" | "cpp" | "h" | "go" | "java"
        | "rb" | "php" | "sql" | "log" | "csv" => AttachmentType::Text,
        // Everything else
        _ => AttachmentType::Binary,
    }
}

/// Maximum file size for attachments (10MB)
pub const MAX_ATTACHMENT_SIZE: u64 = 10 * 1024 * 1024;

/// Check if file is within size limit
pub fn check_size(path: &Path) -> Result<()> {
    let metadata = std::fs::metadata(path)?;
    if metadata.len() > MAX_ATTACHMENT_SIZE {
        anyhow::bail!(
            "File too large: {} bytes (max: {} bytes)",
            metadata.len(),
            MAX_ATTACHMENT_SIZE
        );
    }
    Ok(())
}
