use cursive::traits::{Nameable, Resizable};
use cursive::views::{Dialog, DummyView, EditView, LinearLayout, ScrollView, TextView};
use cursive::Cursive;
use std::path::PathBuf;

use crate::csv::backend;
use crate::csv::session::Session;
use crate::ui::dialogs;

/*
 * ───────────────────────────────────────────────
 *  Viewer state — stored as Cursive user_data
 * ───────────────────────────────────────────────
 */

struct ViewerState {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    session: Session,
}

/*
 * ───────────────────────────────────────────────
 *  Public entry point
 * ───────────────────────────────────────────────
 */

/// Called from csv_workflow after field selection.
pub fn show_inspection_dialog(s: &mut Cursive, path: PathBuf, editable_cols: Vec<usize>) {
    let headers = match backend::read_header(&path) {
        Ok(h) => h,
        Err(e) => {
            dialogs::show_error(s, e);
            return;
        }
    };

    let rows = match backend::read_all_rows(&path) {
        Ok(r) => r,
        Err(e) => {
            dialogs::show_error(s, e);
            return;
        }
    };

    if rows.is_empty() {
        dialogs::show_message(s, "No Data", "No data rows found in this CSV file.");
        return;
    }

    let timestamp = Session::timestamp_now();
    let output_path = Session::make_output_path(&path, &timestamp);
    let total = rows.len();

    let session = Session::new(path, output_path, editable_cols, total, timestamp);

    if let Err(e) = session.save() {
        dialogs::show_error(s, e);
        return;
    }

    let state = ViewerState {
        headers,
        rows,
        session,
    };
    s.set_user_data(state);
    show_current_record(s);
}

/// Restore a previously saved session.
pub fn restore_session(s: &mut Cursive, session: Session) {
    let headers = match backend::read_header(&session.original_path) {
        Ok(h) => h,
        Err(e) => {
            dialogs::show_error(s, e);
            return;
        }
    };

    let rows = match backend::read_all_rows(&session.original_path) {
        Ok(r) => r,
        Err(e) => {
            dialogs::show_error(s, e);
            return;
        }
    };

    if session.current_index >= rows.len() {
        show_completion_summary(s, &session);
        return;
    }

    let state = ViewerState {
        headers,
        rows,
        session,
    };
    s.set_user_data(state);
    show_current_record(s);
}

/*
 * ───────────────────────────────────────────────
 *  Record display
 * ───────────────────────────────────────────────
 */

fn show_current_record(s: &mut Cursive) {
    let (layout, title, has_editable, pending) = {
        let state = match s.user_data::<ViewerState>() {
            Some(st) => st,
            None => return,
        };

        let idx = state.session.current_index;
        let total = state.rows.len();

        if idx >= total {
            let session = state.session.clone();
            show_completion_summary(s, &session);
            return;
        }

        let row = &state.rows[idx];
        let title = format!("Displaying record no {} of {}", idx + 1, total);
        let has_editable = !state.session.editable_cols.is_empty();

        let stats = format!(
            "Saved: {}  |  Discarded: {}  |  Remaining: {}",
            state.session.saved_count,
            state.session.discarded_count,
            total - idx,
        );

        let mut layout = LinearLayout::vertical();

        /* Status bar */
        layout.add_child(TextView::new(stats));
        layout.add_child(DummyView);

        /* Column fields */
        for (col_idx, header) in state.headers.iter().enumerate() {
            let value = row.get(col_idx).cloned().unwrap_or_default();
            let is_editable = state.session.editable_cols.contains(&col_idx);
            let field = build_field_row(header, &value, col_idx, is_editable);
            layout.add_child(field);
        }

        /* Collect pending edits to apply after dialog is mounted */
        let pending: Vec<(usize, String)> = state
            .session
            .pending_edits
            .iter()
            .map(|(k, v)| (*k, v.clone()))
            .collect();

        (layout, title, has_editable, pending)
    };

    let mut dialog = Dialog::new()
        .title(title)
        .content(ScrollView::new(layout).fixed_size((70, 16)));

    if has_editable {
        dialog.add_button("Edit Column", |s| {
            dialogs::show_message(
                s,
                "Editing",
                "Editable fields are marked with [*].\n\
                 Simply type in the field to change its value.",
            );
        });
    }

    dialog.add_button("Save", |s| {
        save_record(s);
    });

    dialog.add_button("Discard", |s| {
        discard_record(s);
    });

    dialog.add_button("Quit", |s| {
        quit_session(s);
    });

    s.add_layer(dialog);

    /* Apply pending edits to EditViews */
    for (col_idx, value) in pending {
        s.call_on_name(&format!("edit_{}", col_idx), |v: &mut EditView| {
            v.set_content(value);
        });
    }
}

fn build_field_row(header: &str, value: &str, col_idx: usize, is_editable: bool) -> LinearLayout {
    let label = if is_editable {
        format!("[*] {}: ", header)
    } else {
        format!("    {}: ", header)
    };

    let mut row = LinearLayout::horizontal();
    row.add_child(TextView::new(label).fixed_width(22));

    if is_editable {
        row.add_child(
            EditView::new()
                .content(value)
                .with_name(format!("edit_{}", col_idx))
                .fixed_width(40),
        );
    } else {
        row.add_child(TextView::new(value).fixed_width(40));
    }

    row
}

/*
 * ───────────────────────────────────────────────
 *  Collect edits from UI
 * ───────────────────────────────────────────────
 */

/// Collects the current values from all editable EditViews.
/// Must be called while the record dialog is on screen.
fn collect_edits(s: &mut Cursive) -> Vec<(usize, String)> {
    /* Copy the editable column list first, then drop the borrow */
    let editable_cols: Vec<usize> = {
        let state = s.user_data::<ViewerState>().unwrap();
        state.session.editable_cols.clone()
    };

    let mut edits = Vec::new();
    for col_idx in editable_cols {
        if let Some(val) = s.call_on_name(&format!("edit_{}", col_idx), |v: &mut EditView| {
            v.get_content().to_string()
        }) {
            edits.push((col_idx, val));
        }
    }

    edits
}

/*
 * ───────────────────────────────────────────────
 *  Record actions
 * ───────────────────────────────────────────────
 */

/// Save current record (with any edits) to the output CSV.
fn save_record(s: &mut Cursive) {
    let edits = collect_edits(s);

    /* Build the output row: original values with edits applied */
    let (row, headers, output_path) = {
        let state = s.user_data::<ViewerState>().unwrap();
        let idx = state.session.current_index;
        let mut row = state.rows[idx].clone();

        for (col_idx, value) in &edits {
            if *col_idx < row.len() {
                row[*col_idx] = value.clone();
            }
        }

        (
            row,
            state.headers.clone(),
            state.session.output_path.clone(),
        )
    };

    /* Append to output file */
    if let Err(e) = backend::append_row(&output_path, &headers, &row) {
        dialogs::show_error(s, e);
        return;
    }

    /* Advance state */
    {
        let state = s.user_data::<ViewerState>().unwrap();
        state.session.saved_count += 1;
        state.session.current_index += 1;
        state.session.pending_edits.clear();
        state.session.last_activity = Session::timestamp_now();

        if let Err(e) = state.session.save() {
            dialogs::show_error(s, e);
            return;
        }
    }

    s.pop_layer();
    show_current_record(s);
}

/// Discard current record — skip it, don't write to output.
fn discard_record(s: &mut Cursive) {
    {
        let state = s.user_data::<ViewerState>().unwrap();
        state.session.discarded_count += 1;
        state.session.current_index += 1;
        state.session.pending_edits.clear();
        state.session.last_activity = Session::timestamp_now();

        if let Err(e) = state.session.save() {
            dialogs::show_error(s, e);
            return;
        }
    }

    s.pop_layer();
    show_current_record(s);
}

/// Save pending edits to session and quit back to main menu.
fn quit_session(s: &mut Cursive) {
    let edits = collect_edits(s);

    {
        let state = s.user_data::<ViewerState>().unwrap();
        state.session.pending_edits.clear();

        for (col_idx, value) in edits {
            state.session.pending_edits.insert(col_idx, value);
        }

        state.session.last_activity = Session::timestamp_now();

        if let Err(e) = state.session.save() {
            dialogs::show_error(s, e);
            return;
        }
    }

    s.pop_layer();
    dialogs::show_message(
        s,
        "Session Saved",
        "Your progress has been saved.\n\
         You can restore it from the main menu.",
    );
}

/*
 * ───────────────────────────────────────────────
 *  Completion
 * ───────────────────────────────────────────────
 */

fn show_completion_summary(s: &mut Cursive, session: &Session) {
    let msg = format!(
        "All records have been processed!\n\n\
         Total records:  {}\n\
         Saved:          {}\n\
         Discarded:      {}\n\n\
         Output file:\n  {}\n\n\
         Session started: {}\n\
         Completed:       {}",
        session.total_rows,
        session.saved_count,
        session.discarded_count,
        session.output_path.display(),
        session.started_at,
        Session::timestamp_now(),
    );

    let _ = Session::delete(&session.original_path);

    dialogs::show_message(s, "Complete", msg);
}
