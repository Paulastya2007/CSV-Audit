use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Persistent session state, serialized to JSON beside the original CSV.
///
/// `edits` maps row_index -> (col_index -> edited_value) for rows the user
/// modified but hasn't saved/discarded yet (i.e. the "current" row if they
/// quit mid-inspection).  Saved/discarded rows are tracked by `current_index`
/// — everything before that index has already been processed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub original_path: PathBuf,
    pub output_path: PathBuf,
    pub editable_cols: Vec<usize>,
    pub current_index: usize,
    pub total_rows: usize,
    pub saved_count: usize,
    pub discarded_count: usize,
    pub pending_edits: HashMap<usize, String>,
    pub started_at: String,
    pub last_activity: String,
}

impl Session {
    /// Build the session file path from the original CSV path.
    /// e.g. `/data/passwords.csv` → `/data/passwords.csv.session.json`
    pub fn session_path(original: &Path) -> PathBuf {
        let mut p = original.as_os_str().to_owned();
        p.push(".session.json");
        PathBuf::from(p)
    }

    /// Build the output CSV path with a timestamp.
    /// e.g. `passwords.csv` → `passwords_filtered_2025-06-20_14-30-00.csv`
    pub fn make_output_path(original: &Path, timestamp: &str) -> PathBuf {
        let stem = original.file_stem().unwrap_or_default().to_string_lossy();

        let parent = original.parent().unwrap_or_else(|| Path::new("."));
        let filename = format!("{}_filtered_{}.csv", stem, timestamp);

        parent.join(filename)
    }

    /// Create a fresh session.
    pub fn new(
        original_path: PathBuf,
        output_path: PathBuf,
        editable_cols: Vec<usize>,
        total_rows: usize,
        timestamp: String,
    ) -> Self {
        Session {
            original_path,
            output_path,
            editable_cols,
            current_index: 0,
            total_rows,
            saved_count: 0,
            discarded_count: 0,
            pending_edits: HashMap::new(),
            started_at: timestamp.clone(),
            last_activity: timestamp,
        }
    }

    /// Save session state to disk.
    pub fn save(&self) -> Result<(), String> {
        let path = Self::session_path(&self.original_path);
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize session: {}", e))?;

        std::fs::write(&path, json)
            .map_err(|e| format!("Failed to write session file {}: {}", path.display(), e))?;

        Ok(())
    }

    /// Load session state from disk. Returns None if file doesn't exist.
    pub fn load(original_path: &Path) -> Result<Option<Self>, String> {
        let path = Self::session_path(original_path);

        if !path.exists() {
            return Ok(None);
        }

        let json = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read session file {}: {}", path.display(), e))?;

        let session: Session = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse session file: {}", e))?;

        Ok(Some(session))
    }

    /// Delete session file from disk.
    pub fn delete(original_path: &Path) -> Result<(), String> {
        let path = Self::session_path(original_path);

        if path.exists() {
            std::fs::remove_file(&path)
                .map_err(|e| format!("Failed to delete session file: {}", e))?;
        }

        Ok(())
    }

    /// Get a formatted timestamp string.
    pub fn timestamp_now() -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();

        let secs = now.as_secs();

        /* Manual UTC breakdown — no chrono dependency needed */
        let days = secs / 86400;
        let time_of_day = secs % 86400;
        let hours = time_of_day / 3600;
        let minutes = (time_of_day % 3600) / 60;
        let seconds = time_of_day % 60;

        /* Compute year/month/day from days since epoch (1970-01-01) */
        let (year, month, day) = days_to_ymd(days);

        format!(
            "{:04}-{:02}-{:02}_{:02}-{:02}-{:02}",
            year, month, day, hours, minutes, seconds
        )
    }
}

/// Convert days since Unix epoch to (year, month, day).
fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    let mut year = 1970u64;

    loop {
        let year_days = if is_leap(year) { 366 } else { 365 };
        if days < year_days {
            break;
        }
        days -= year_days;
        year += 1;
    }

    let leap = is_leap(year);
    let month_days: [u64; 12] = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];

    let mut month = 1u64;
    for &md in &month_days {
        if days < md {
            break;
        }
        days -= md;
        month += 1;
    }

    (year, month, days + 1)
}
fn is_leap(y: u64) -> bool {
    (y.is_multiple_of(4) && !y.is_multiple_of(100)) || y.is_multiple_of(400)
}
