use std::path::{Path, PathBuf};

/// Metadata about a discovered CSV file.
#[derive(Debug, Clone)]
pub struct CsvFile {
    /// Absolute path to the file.
    pub path: PathBuf,
    /// File size in bytes.
    pub size_bytes: u64,
    /// Number of data rows (excluding the header, if any).
    pub row_count: usize,
}

impl CsvFile {
    /// Display label shown in the file picker (relative path + row count).
    pub fn label(&self, root: &Path) -> String {
        let rel = self
            .path
            .strip_prefix(root)
            .unwrap_or(&self.path)
            .to_string_lossy()
            .into_owned();
        format!(
            "{} ({} rows, {} bytes)",
            rel, self.row_count, self.size_bytes
        )
    }
}

/// Opens and inspects a CSV file, returning a populated [`CsvFile`].
pub fn inspect_csv(path: &Path) -> Result<CsvFile, String> {
    let size_bytes = std::fs::metadata(path)
        .map(|m| m.len())
        .map_err(|e| format!("Cannot read metadata for {}: {}", path.display(), e))?;

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_path(path)
        .map_err(|e| format!("Cannot open CSV {}: {}", path.display(), e))?;

    let row_count = reader.records().count();

    Ok(CsvFile {
        path: path.to_path_buf(),
        size_bytes,
        row_count,
    })
}

/// Read header row from a CSV file.
pub fn read_header(path: &Path) -> Result<Vec<String>, String> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_path(path)
        .map_err(|e| format!("Cannot open CSV {}: {}", path.display(), e))?;

    let headers = rdr
        .headers()
        .map_err(|e| format!("Failed to read headers from {}: {}", path.display(), e))?
        .iter()
        .map(|s| s.to_string())
        .collect();

    Ok(headers)
}

/// Reads CSV rows, optionally selecting specific column indexes.
pub fn read_rows(path: &Path, selected_cols: Option<&[usize]>) -> Result<Vec<Vec<String>>, String> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_path(path)
        .map_err(|e| format!("Cannot open CSV {}: {}", path.display(), e))?;

    let mut rows = Vec::new();

    for result in rdr.records() {
        let record = result.map_err(|e| format!("Error reading CSV {}: {}", path.display(), e))?;
        let row: Vec<String> = match selected_cols {
            Some(cols) => cols
                .iter()
                .filter_map(|&i| record.get(i).map(|s| s.to_string()))
                .collect(),
            None => record.iter().map(|s| s.to_string()).collect(),
        };
        rows.push(row);
    }

    Ok(rows)
}

/// Read all data rows from a CSV file.
pub fn read_all_rows(path: &Path) -> Result<Vec<Vec<String>>, String> {
    read_rows(path, None)
}

/// Append a single row to an existing CSV file.
/// If the file does not exist, create it and write the header first.
pub fn append_row(path: &Path, headers: &[String], row: &[String]) -> Result<(), String> {
    let file_exists = path.exists();

    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("Cannot open output file {}: {}", path.display(), e))?;

    let mut wtr = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(file);

    if !file_exists {
        wtr.write_record(headers)
            .map_err(|e| format!("Failed to write header: {}", e))?;
    }

    wtr.write_record(row)
        .map_err(|e| format!("Failed to write row: {}", e))?;

    wtr.flush()
        .map_err(|e| format!("Failed to flush output: {}", e))?;

    Ok(())
}
