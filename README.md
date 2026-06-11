# CSV Audit

A terminal-based CSV inspection and sanitization tool built in Rust using the [Cursive](https://github.com/gyscos/cursive) TUI framework.

CSV Audit lets you walk through CSV records one at a time, review each field, optionally edit selected columns, and selectively approve or discard records into a clean output file — all without ever modifying the original data.

Built for security professionals, data analysts, and anyone who needs to manually review and sanitize CSV dumps such as leaked credential databases, exported password lists, or audit logs.

---

## Features

- **Interactive record-by-record review** — navigate through CSV data one row at a time in a clean terminal UI
- **Selective column editing** — mark specific columns as editable (e.g., update a password field) while keeping the rest read-only
- **Approve or discard** — save approved records to a timestamped output file, skip unwanted ones
- **Session persistence** — quit anytime and resume later from exactly where you left off, including unsaved edits
- **Non-destructive** — the original CSV file is never modified
- **Timestamped output** — output files are named with the date and time to avoid overwrites
- **Auto-discovery** — scans the current directory and one level of subdirectories for CSV files
- **Manual path entry** — type or paste a full path to open any CSV file on the system

---


### Run directly

```bash
cargo run
```

### Run tests

```bash
cargo test
```

---

## Usage

### 1. Open a CSV file

Launch the app and select **Open CSV File**. The tool scans your current directory for `.csv` files and lists them. You can also type a full path manually.

### 2. Select editable columns

After opening a file, you're shown all column headers with checkboxes. Check the columns you want to be able to edit during review (e.g., `password`, `status`). Select **None** if you only want to review without editing.

### 3. Review records

Records are displayed one at a time with all columns visible:

- **Non-editable columns** are shown as plain text
- **Editable columns** are marked with `[*]` and displayed as input fields you can type into

### 4. Save or Discard

| Button | Action |
|--------|--------|
| **Save** | Approves the record (with any edits) and appends it to the output CSV |
| **Discard** | Skips the record — it will not appear in the output CSV |
| **Edit Column** | Reminder that fields marked `[*]` are editable inline |
| **Quit** | Saves current progress to a session file and returns to main menu |

### 5. Resume a session

Select **Restore Previous Session** from the main menu. The tool finds saved session files, shows you the progress summary, and lets you pick up exactly where you left off.

### 6. Completion

After all records are processed, a summary is shown:

```
All records have been processed!

Total records:  10
Saved:          7
Discarded:      3

Output file:
  /data/accounts_filtered_2025-06-20_14-30-00.csv

Session started: 2025-06-20_14-30-00
Completed:       2025-06-20_14-45-12
```

---

## File Outputs

| File | Location | Purpose |
|------|----------|---------|
| `original.csv` | Wherever the user placed it | **Never modified** |
| `original_filtered_YYYY-MM-DD_HH-MM-SS.csv` | Same directory as original | Approved records with any edits applied |
| `original.csv.session.json` | Same directory as the executable | Session state for resume capability |

The session file is automatically deleted when all records have been processed.

---

## Architecture

```
src/
├── main.rs                 Entry point — initializes Cursive and shows main menu
├── lib.rs                  Crate root — exposes csv and ui modules
├── csv/
│   ├── mod.rs              Module declarations
│   ├── backend.rs          CSV I/O operations (read, write, append)
│   ├── helpdir.rs          Filesystem scanner (finds CSV files)
│   └── session.rs          Session persistence (save/load/delete JSON)
└── ui/
    ├── mod.rs              Module declarations
    ├── menu.rs             Main menu and session restore UI
    ├── dialogs.rs          Reusable dialog helpers (message, error, confirm)
    ├── csv_workflow.rs     File picker and editable column selector
    └── csv_work.rs         Record viewer, editor, and save/discard logic
```

### Data Flow

```
main.rs
  │
  ▼
menu.rs ─── show_main_menu()
  │
  ├── "Open CSV"
  │     │
  │     ▼
  │   csv_workflow.rs ─── open_csv_workflow()
  │     │   helpdir scans for .csv files
  │     │   user picks a file
  │     │
  │     ▼
  │   csv_workflow.rs ─── load_and_select_fields()
  │     │   backend reads headers
  │     │   user selects editable columns
  │     │
  │     ▼
  │   csv_work.rs ─── show_inspection_dialog()
  │     │   backend reads ALL rows into memory
  │     │   session created and saved to disk
  │     │   ViewerState stored in Cursive user_data
  │     │
  │     ▼
  │   csv_work.rs ─── show_current_record()  ◄──────┐
  │     │   displays one record                      │
  │     │   editable fields shown as EditViews       │
  │     │                                            │
  │     ├── Save ───────────────────────────────────►│
  │     │     collect edits from EditViews           │
  │     │     apply edits to row copy                │
  │     │     append to output CSV                   │
  │     │     advance index, save session            │
  │     │                                            │
  │     ├── Discard ────────────────────────────────►│
  │     │     advance index, save session            │
  │     │                                            │
  │     ├── Quit                                     │
  │     │     save pending edits to session          │
  │     │     return to main menu                    │
  │     │                                            │
  │     └── All done                                 │
  │           show summary, delete session           │
  │
  └── "Restore Session"
        │
        ▼
      menu.rs ─── find_session_files()
        │   scans exe directory for .session.json
        │   user selects session to restore
        │
        ▼
      csv_work.rs ─── restore_session()
        │   reloads CSV data from original file
        │   resumes from saved current_index
        │   restores pending edits to EditViews
        │
        └── show_current_record() ──────────────────►(same loop)
```

### Layer Responsibilities

| Layer | Responsibility | Dependencies |
|-------|---------------|--------------|
| `backend.rs` | Raw CSV read/write operations | `csv` crate, filesystem |
| `helpdir.rs` | Finding CSV files on disk | filesystem only |
| `session.rs` | Session state serialization | `serde`, `serde_json`, filesystem |
| `dialogs.rs` | Reusable UI popups | `cursive` only |
| `csv_workflow.rs` | File selection and column configuration | `backend`, `dialogs` |
| `csv_work.rs` | Record viewing, editing, and approval loop | `backend`, `session`, `dialogs` |
| `menu.rs` | Top-level navigation and session restore | all UI modules, `session` |

### State Management

The application stores a `ViewerState` struct in Cursive's `user_data`:

```rust
struct ViewerState {
    headers: Vec<String>,       // column names from the CSV header
    rows: Vec<Vec<String>>,     // ALL data rows loaded into memory
    session: Session,           // current progress and configuration
}
```

All rows are loaded into memory at startup. This means:

- **Fast navigation** — accessing any row is an instant index lookup
- **Memory proportional to file size** — a 100MB CSV uses ~100MB of RAM
- **Not suitable for very large files** — files over ~500MB may cause issues

### Session Persistence

Session state is serialized to JSON:

```json
{
  "original_path": "/data/accounts.csv",
  "output_path": "/data/accounts_filtered_2025-06-20_14-30-00.csv",
  "editable_cols": [3, 5],
  "current_index": 42,
  "total_rows": 100,
  "saved_count": 30,
  "discarded_count": 12,
  "pending_edits": {
    "3": "updated_password"
  },
  "started_at": "2025-06-20_14-30-00",
  "last_activity": "2025-06-20_15-12-33"
}
```

The session file is saved:
- After every Save or Discard action
- When the user quits mid-session
- Deleted automatically when all records are processed

---

## Dependencies
```toml
csv = "1.4.0"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
cursive = { version = "0.21.1", default-features = false, features = ["crossterm-backend"] }
```

---

## Limitations

- **All rows loaded into RAM** — the entire CSV is read into memory at startup. Files larger than available RAM will cause problems. A future version may implement row-level seeking with a byte-offset index.
- **Single-threaded** — all I/O and UI runs on a single thread. The UI may briefly freeze when opening very large files.
- **No undo** — once a record is saved or discarded, it cannot be revisited in the current session.
- **UTF-8 only** — CSV files must be UTF-8 encoded.

---

## Future Improvements

- [ ] Row-level byte-offset indexing for constant-memory operation
- [ ] Background I/O thread with loading indicator
- [ ] Undo/go-back to revisit previous records
- [ ] Search and filter within records
- [ ] Bulk approve/discard by pattern matching
- [ ] Export session summary as a report
- [ ] Custom delimiter support (TSV, pipe-delimited, etc.)

---

## Contributing

Contributions are welcome. Please open an issue first to discuss what you'd like to change.
