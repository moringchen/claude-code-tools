use serde::Deserialize;
use std::path::PathBuf;

/// History entry from ~/.claude/history.jsonl
#[derive(Debug, Deserialize)]
struct HistoryEntry {
    #[serde(rename = "sessionId")]
    session_id: String,
    display: String,
}

/// Claude Code session file format (e.g., ~/.claude/sessions/{pid}.json)
#[derive(Debug, Deserialize, Default)]
pub struct SessionMetadata {
    pub pid: u32,
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub cwd: String,
    /// Task name from session
    pub name: Option<String>,
    /// Session kind: "interactive", "sidechain", etc.
    pub kind: Option<String>,
}

impl SessionMetadata {
    /// Extract the task title from session name
    pub fn extract_title(&self) -> Option<String> {
        self.name.clone().filter(|t| !t.is_empty())
    }

    /// Check if this is a sidechain (background) session
    pub fn is_sidechain(&self) -> bool {
        self.kind.as_deref() == Some("sidechain")
    }
}

/// Read session metadata using pid
pub fn read_session_metadata_by_pid(pid: u32) -> Option<SessionMetadata> {
    let session_path = find_session_file_by_pid(pid)?;
    read_session_file(&session_path)
}

/// Find session file by pid (e.g., ~/.claude/sessions/12345.json)
fn find_session_file_by_pid(pid: u32) -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;

    // Try standard Claude Code session locations with pid.json format
    let possible_paths = [
        format!("{}/.claude/sessions/{}.json", home, pid),
        format!("{}/.claude-code/sessions/{}.json", home, pid),
    ];

    for path in &possible_paths {
        let path_buf = PathBuf::from(path);
        if path_buf.exists() {
            return Some(path_buf);
        }
    }

    None
}

/// Find session file by session_id (search all session files)
pub fn find_session_file_by_session_id(session_id: &str) -> Option<(PathBuf, SessionMetadata)> {
    let home = std::env::var("HOME").ok()?;

    let base_dirs = [
        format!("{}/.claude/sessions", home),
        format!("{}/.claude-code/sessions", home),
    ];

    for base_dir in &base_dirs {
        let base = PathBuf::from(base_dir);
        if !base.exists() {
            continue;
        }

        // Read all .json files in the sessions directory
        if let Ok(entries) = std::fs::read_dir(&base) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension()?.to_str()? != "json" {
                    continue;
                }

                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(metadata) = serde_json::from_str::<SessionMetadata>(&content) {
                        if metadata.session_id == session_id {
                            return Some((path, metadata));
                        }
                    }
                }
            }
        }
    }

    None
}

/// Read and parse session file
fn read_session_file(path: &PathBuf) -> Option<SessionMetadata> {
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Extract title from cwd as fallback
pub fn extract_title_from_cwd(cwd: Option<&str>) -> String {
    cwd.and_then(|path| {
        path.split('/')
            .filter(|s| !s.is_empty())
            .last()
            .map(|s| s.to_string())
    })
    .unwrap_or_else(|| "Claude Task".to_string())
}

/// Read first prompt from history.jsonl for a given session_id
fn read_first_prompt_from_history(session_id: &str) -> Option<String> {
    let home = std::env::var("HOME").ok()?;
    let history_path = format!("{}/.claude/history.jsonl", home);

    let content = std::fs::read_to_string(&history_path).ok()?;

    // Read lines from bottom to top (most recent first)
    for line in content.lines().rev() {
        if let Ok(entry) = serde_json::from_str::<HistoryEntry>(line) {
            if entry.session_id == session_id && !entry.display.is_empty() {
                return Some(entry.display);
            }
        }
    }

    None
}

/// Get the best title for a task using pid
pub fn get_task_title_by_pid(pid: u32, cwd: Option<&str>, fallback_title: &str) -> String {
    // Try to read from session metadata by pid
    if let Some(metadata) = read_session_metadata_by_pid(pid) {
        // Skip sidechain sessions
        if metadata.is_sidechain() {
            return String::new(); // Empty title signals it should be filtered
        }

        // First try to get first prompt from history.jsonl using session_id
        // (user's actual first prompt is preferred over session name)
        if let Some(prompt) = read_first_prompt_from_history(&metadata.session_id) {
            return truncate_title(&prompt);
        }

        // Then try the name field from session metadata
        if let Some(title) = metadata.extract_title() {
            return truncate_title(&title);
        }
    }

    // Try to extract from cwd
    if let Some(cwd_title) = cwd.and_then(|p| p.split('/').filter(|s| !s.is_empty()).last()) {
        return cwd_title.to_string();
    }

    // Use fallback with truncation
    truncate_title(fallback_title)
}

/// Get the best title for a task using session_id
pub fn get_task_title_by_session_id(
    session_id: &str,
    cwd: Option<&str>,
    fallback_title: &str,
) -> String {
    // Try to find session file by session_id
    if let Some((_, metadata)) = find_session_file_by_session_id(session_id) {
        // Skip sidechain sessions
        if metadata.is_sidechain() {
            return String::new();
        }

        // First try to get first prompt from history.jsonl
        // (user's actual first prompt is preferred over session name)
        if let Some(prompt) = read_first_prompt_from_history(session_id) {
            return truncate_title(&prompt);
        }

        // Then try the name field from session metadata
        if let Some(title) = metadata.extract_title() {
            return truncate_title(&title);
        }
    }

    // Try to get first prompt from history.jsonl (if no session file found)
    if let Some(prompt) = read_first_prompt_from_history(session_id) {
        return truncate_title(&prompt);
    }

    // Try to extract from cwd
    if let Some(cwd_title) = cwd.and_then(|p| p.split('/').filter(|s| !s.is_empty()).last()) {
        return cwd_title.to_string();
    }

    // Use fallback with truncation
    truncate_title(fallback_title)
}

/// Clean and truncate title to reasonable length (50 chars)
/// Removes /xxx:xxx prefix and truncates if needed
fn truncate_title(title: &str) -> String {
    let trimmed = title.trim();

    // Remove /xxx:xxx prefix pattern (e.g., "/superpowers:brainstorming message" -> "message")
    let cleaned = if trimmed.starts_with('/') {
        if let Some(space_pos) = trimmed.find(' ') {
            let prefix = &trimmed[..space_pos];
            if prefix.contains(':') {
                &trimmed[space_pos + 1..].trim_start()
            } else {
                trimmed
            }
        } else {
            trimmed
        }
    } else {
        trimmed
    };

    if cleaned.chars().count() > 50 {
        let truncated: String = cleaned.chars().take(50).collect();
        format!("{}...", truncated)
    } else {
        cleaned.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_title_from_name() {
        let meta = SessionMetadata {
            pid: 123,
            session_id: "test-id".to_string(),
            cwd: "/workspace".to_string(),
            name: Some("My Task".to_string()),
            kind: Some("interactive".to_string()),
        };
        assert_eq!(meta.extract_title(), Some("My Task".to_string()));
    }

    #[test]
    fn test_is_sidechain() {
        let meta = SessionMetadata {
            pid: 123,
            session_id: "test-id".to_string(),
            cwd: "/workspace".to_string(),
            name: None,
            kind: Some("sidechain".to_string()),
        };
        assert!(meta.is_sidechain());
    }

    #[test]
    fn test_is_not_sidechain() {
        let meta = SessionMetadata {
            pid: 123,
            session_id: "test-id".to_string(),
            cwd: "/workspace".to_string(),
            name: None,
            kind: Some("interactive".to_string()),
        };
        assert!(!meta.is_sidechain());
    }
}
