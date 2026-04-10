//! File system commands for project navigation

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri_plugin_dialog::DialogExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<FileNode>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extension: Option<String>,
}

/// Patterns to ignore when building file tree
const IGNORE_PATTERNS: &[&str] = &[
    "node_modules",
    ".git",
    "target",
    "dist",
    "build",
    ".next",
    "__pycache__",
    ".venv",
    "venv",
    ".idea",
    ".vscode",
    "*.pyc",
    "*.pyo",
    ".DS_Store",
    "Thumbs.db",
];

fn should_ignore(name: &str) -> bool {
    IGNORE_PATTERNS.iter().any(|pattern| {
        if pattern.starts_with('*') {
            name.ends_with(&pattern[1..])
        } else {
            name == *pattern
        }
    })
}

fn build_tree(path: &Path, depth: usize, max_depth: usize) -> Option<FileNode> {
    let name = path.file_name()?.to_string_lossy().to_string();

    if should_ignore(&name) {
        return None;
    }

    let is_dir = path.is_dir();
    let extension = if !is_dir {
        path.extension().map(|e| e.to_string_lossy().to_string())
    } else {
        None
    };

    let size = if !is_dir {
        std::fs::metadata(path).ok().map(|m| m.len())
    } else {
        None
    };

    let children = if is_dir && depth < max_depth {
        match std::fs::read_dir(path) {
            Ok(entries) => {
                let mut children: Vec<FileNode> = entries
                    .filter_map(|e| e.ok())
                    .filter_map(|e| build_tree(&e.path(), depth + 1, max_depth))
                    .collect();

                // Sort: directories first, then alphabetically
                children.sort_by(|a, b| {
                    match (a.is_dir, b.is_dir) {
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                    }
                });

                if children.is_empty() {
                    None
                } else {
                    Some(children)
                }
            }
            Err(_) => None,
        }
    } else {
        None
    };

    Some(FileNode {
        name,
        path: path.to_string_lossy().to_string(),
        is_dir,
        children,
        size,
        extension,
    })
}

/// Load file tree for a project directory
#[tauri::command]
pub async fn load_file_tree(path: String, max_depth: Option<usize>) -> Result<Vec<FileNode>, String> {
    let path = PathBuf::from(&path);

    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }

    if !path.is_dir() {
        return Err(format!("Path is not a directory: {}", path.display()));
    }

    let max_depth = max_depth.unwrap_or(3);

    match std::fs::read_dir(&path) {
        Ok(entries) => {
            let mut nodes: Vec<FileNode> = entries
                .filter_map(|e| e.ok())
                .filter_map(|e| build_tree(&e.path(), 0, max_depth))
                .collect();

            // Sort: directories first, then alphabetically
            nodes.sort_by(|a, b| {
                match (a.is_dir, b.is_dir) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                }
            });

            Ok(nodes)
        }
        Err(e) => Err(format!("Failed to read directory: {}", e)),
    }
}

/// Read file content
#[tauri::command]
pub async fn read_file(path: String) -> Result<String, String> {
    let path = PathBuf::from(&path);

    if !path.exists() {
        return Err(format!("File does not exist: {}", path.display()));
    }

    if !path.is_file() {
        return Err(format!("Path is not a file: {}", path.display()));
    }

    // Check file size (limit to 10MB)
    let metadata = std::fs::metadata(&path)
        .map_err(|e| format!("Failed to get file metadata: {}", e))?;

    if metadata.len() > 10 * 1024 * 1024 {
        return Err("File is too large (>10MB)".to_string());
    }

    std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file: {}", e))
}

/// Get file preview (first N lines)
#[tauri::command]
pub async fn get_file_preview(path: String, lines: Option<usize>) -> Result<String, String> {
    let content = read_file(path).await?;
    let lines = lines.unwrap_or(50);

    let preview: String = content
        .lines()
        .take(lines)
        .collect::<Vec<_>>()
        .join("\n");

    Ok(preview)
}

/// Open folder selection dialog
#[tauri::command]
pub async fn select_project_folder(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use std::sync::mpsc;
    let (tx, rx) = mpsc::channel();

    app.dialog()
        .file()
        .set_title("Select Project Folder")
        .pick_folder(move |folder| {
            let _ = tx.send(folder.map(|p| p.to_string()));
        });

    match rx.recv() {
        Ok(result) => Ok(result),
        Err(_) => Ok(None),
    }
}
