//! Unified diff generation
//!
//! Generates unified diff format for file changes.

use similar::{ChangeTag, TextDiff};

/// Generate unified diff between two strings
pub fn generate_unified_diff(old: &str, new: &str, path: &str) -> String {
    let diff = TextDiff::from_lines(old, new);

    let mut result = String::new();

    // Header
    result.push_str(&format!("--- a/{}\n", path));
    result.push_str(&format!("+++  b/{}\n", path));

    // Generate hunks using unified_diff to_string which handles formatting
    for hunk in diff.unified_diff().context_radius(3).iter_hunks() {
        // Get hunk header info
        let header = hunk.header();
        result.push_str(&format!(
            "@@ -{},{} +{},{} @@\n",
            header.old_range.start + 1,
            header.old_range.len,
            header.new_range.start + 1,
            header.new_range.len
        ));

        for change in hunk.iter_changes() {
            let sign = match change.tag() {
                ChangeTag::Delete => '-',
                ChangeTag::Insert => '+',
                ChangeTag::Equal => ' ',
            };
            result.push(sign);
            result.push_str(change.value());
            if !change.value().ends_with('\n') {
                result.push('\n');
            }
        }
    }

    result
}

/// Generate a compact summary of changes
pub fn diff_summary(old: &str, new: &str) -> DiffSummary {
    let diff = TextDiff::from_lines(old, new);

    let mut added = 0;
    let mut removed = 0;
    let mut changed_lines = Vec::new();

    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Insert => {
                added += 1;
                if let Some(line) = change.new_index() {
                    changed_lines.push(line + 1);
                }
            }
            ChangeTag::Delete => {
                removed += 1;
            }
            ChangeTag::Equal => {}
        }
    }

    DiffSummary {
        lines_added: added,
        lines_removed: removed,
        changed_lines,
    }
}

/// Summary of diff changes
#[derive(Debug, Clone)]
pub struct DiffSummary {
    pub lines_added: usize,
    pub lines_removed: usize,
    pub changed_lines: Vec<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_diff() {
        let old = "line 1\nline 2\nline 3\n";
        let new = "line 1\nmodified line 2\nline 3\n";

        let diff = generate_unified_diff(old, new, "test.txt");

        assert!(diff.contains("--- a/test.txt"));
        assert!(diff.contains("+++ b/test.txt"));
        assert!(diff.contains("-line 2"));
        assert!(diff.contains("+modified line 2"));
    }

    #[test]
    fn test_diff_summary() {
        let old = "line 1\nline 2\nline 3\n";
        let new = "line 1\nnew line\nline 3\nextra line\n";

        let summary = diff_summary(old, new);

        assert_eq!(summary.lines_added, 2);
        assert_eq!(summary.lines_removed, 1);
    }

    #[test]
    fn test_empty_diff() {
        let content = "same content\n";
        let diff = generate_unified_diff(content, content, "test.txt");

        // Should only have headers, no changes
        assert!(diff.contains("--- a/test.txt"));
        assert!(!diff.contains("-same"));
        assert!(!diff.contains("+same"));
    }

    #[test]
    fn test_new_file_diff() {
        let old = "";
        let new = "new content\nmore lines\n";

        let diff = generate_unified_diff(old, new, "new_file.txt");

        assert!(diff.contains("+new content"));
        assert!(diff.contains("+more lines"));
    }
}
