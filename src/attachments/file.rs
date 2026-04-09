//! File attachment handling

use anyhow::Result;
use std::path::Path;

/// Read a text file as attachment
pub fn read_file_attachment(path: &Path) -> Result<FileAttachment> {
    let content = std::fs::read_to_string(path)?;
    let filename = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let mime_type = mime_guess::from_path(path)
        .first_or_text_plain()
        .to_string();

    Ok(FileAttachment {
        filename,
        mime_type,
        content,
        size: content.len(),
    })
}

/// A file attachment
#[derive(Debug, Clone)]
pub struct FileAttachment {
    pub filename: String,
    pub mime_type: String,
    pub content: String,
    pub size: usize,
}

impl FileAttachment {
    /// Format for inclusion in prompt
    pub fn to_prompt_format(&self) -> String {
        format!(
            "<file name=\"{}\" type=\"{}\">\n{}\n</file>",
            self.filename, self.mime_type, self.content
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_format() {
        let attachment = FileAttachment {
            filename: "test.txt".to_string(),
            mime_type: "text/plain".to_string(),
            content: "Hello, World!".to_string(),
            size: 13,
        };

        let formatted = attachment.to_prompt_format();
        assert!(formatted.contains("test.txt"));
        assert!(formatted.contains("Hello, World!"));
    }
}
