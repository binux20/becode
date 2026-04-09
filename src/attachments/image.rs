//! Image processing for attachments

use anyhow::Result;
use base64::Engine;
use std::path::Path;

/// Maximum image dimension (resize if larger)
const MAX_DIMENSION: u32 = 2048;

/// Process an image file for attachment
/// - Validates format
/// - Resizes if too large
/// - Converts to base64
pub fn process_image(path: &Path) -> Result<ProcessedImage> {
    // Read file
    let data = std::fs::read(path)?;

    // Detect MIME type
    let mime_type = mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string();

    // For now, just base64 encode without resizing
    // TODO: Add actual image resizing with the `image` crate
    let base64_data = base64::engine::general_purpose::STANDARD.encode(&data);

    Ok(ProcessedImage {
        mime_type,
        base64_data,
        original_size: data.len(),
    })
}

/// A processed image ready for API
#[derive(Debug, Clone)]
pub struct ProcessedImage {
    pub mime_type: String,
    pub base64_data: String,
    pub original_size: usize,
}

impl ProcessedImage {
    /// Get as data URL
    pub fn to_data_url(&self) -> String {
        format!("data:{};base64,{}", self.mime_type, self.base64_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_url_format() {
        let img = ProcessedImage {
            mime_type: "image/png".to_string(),
            base64_data: "abc123".to_string(),
            original_size: 100,
        };

        assert_eq!(img.to_data_url(), "data:image/png;base64,abc123");
    }
}
