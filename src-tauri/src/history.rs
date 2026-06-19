use crate::config::config_dir;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub source_text: String,
    pub translated_text: String,
    pub source_lang: String,
    pub target_lang: String,
    pub provider: String,
    pub model: String,
    pub mode: String,
    pub tone: String,
    pub preset_id: String,
    pub latency_ms: u128,
}

fn history_path() -> PathBuf {
    config_dir().join("history.jsonl")
}

pub fn load_history() -> Vec<HistoryEntry> {
    let path = history_path();
    if !path.exists() {
        return Vec::new();
    }

    let data = match std::fs::read_to_string(&path) {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };

    data.lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect()
}

pub fn save_history_entry(entry: &HistoryEntry) -> anyhow::Result<()> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir)?;
    let path = history_path();

    let mut line = serde_json::to_string(entry)?;
    line.push('\n');

    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    file.write_all(line.as_bytes())?;
    Ok(())
}

pub fn delete_history_entry(id: &str) -> anyhow::Result<()> {
    let entries = load_history();
    let filtered: Vec<_> = entries.into_iter().filter(|e| e.id != id).collect();

    let path = history_path();
    let lines: Vec<String> = filtered
        .iter()
        .filter_map(|e| serde_json::to_string(e).ok())
        .collect();
    std::fs::write(path, lines.join("\n") + if lines.is_empty() { "" } else { "\n" })?;
    Ok(())
}

pub fn clear_history() -> anyhow::Result<()> {
    let path = history_path();
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}
