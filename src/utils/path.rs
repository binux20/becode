//! Path utilities for safe path handling

use std::path::{Path, PathBuf};

/// Normalize path separators to forward slashes
pub fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Join paths and normalize
pub fn join_normalized(base: &Path, relative: &str) -> PathBuf {
    base.join(relative.replace('\\', "/"))
}

/// Check if a path is safe (doesn't escape via ..)
pub fn is_safe_path(base: &Path, target: &Path) -> bool {
    // Resolve both paths
    let base_canonical = match base.canonicalize() {
        Ok(p) => p,
        Err(_) => return false,
    };

    let target_canonical = match target.canonicalize() {
        Ok(p) => p,
        Err(_) => {
            // If target doesn't exist, check parent
            if let Some(parent) = target.parent() {
                match parent.canonicalize() {
                    Ok(p) => p.join(target.file_name().unwrap_or_default()),
                    Err(_) => return false,
                }
            } else {
                return false;
            }
        }
    };

    target_canonical.starts_with(&base_canonical)
}

/// Get relative path from base to target
pub fn relative_path(base: &Path, target: &Path) -> Option<PathBuf> {
    target.strip_prefix(base).ok().map(|p| p.to_path_buf())
}

/// Sanitize a filename (remove unsafe characters)
pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_normalize_path() {
        let path = Path::new("src\\main\\file.rs");
        assert_eq!(normalize_path(path), "src/main/file.rs");
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("file:name.txt"), "file_name.txt");
        assert_eq!(sanitize_filename("path/to/file"), "path_to_file");
        assert_eq!(sanitize_filename("normal.txt"), "normal.txt");
    }

    #[test]
    fn test_is_safe_path() {
        let base = env::current_dir().unwrap();
        let safe = base.join("src");
        let unsafe_path = base.join("..").join("..").join("etc");

        // Safe path within base
        assert!(is_safe_path(&base, &safe) || !safe.exists());

        // Unsafe path escaping base (if it exists)
        if unsafe_path.exists() {
            assert!(!is_safe_path(&base, &unsafe_path));
        }
    }
}
