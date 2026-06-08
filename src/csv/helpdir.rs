use std::fs;
use std::path::{Path, PathBuf};

/// Searches for `.csv` files in the given root directory and in every
/// immediate subdirectory of that root (one level deep).
///
/// Returns a sorted `Vec<PathBuf>` of every `.csv` file found.
pub fn find_csv_files(root: &Path) -> Vec<PathBuf> {
    let mut results: Vec<PathBuf> = Vec::new();

    // Scan the root itself
    scan_dir(root, &mut results);

    // Scan each immediate subdirectory (one level deep)
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                scan_dir(&path, &mut results);
            }
        }
    }

    results.sort();
    results
}

/// Adds every `.csv` file found directly inside `dir` to `out`.
fn scan_dir(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file()
                && matches!(
                    path.extension(),
                    Some(ext) if ext.eq_ignore_ascii_case("csv")
                )
            {
                out.push(path);
            }
        }
    }
}
