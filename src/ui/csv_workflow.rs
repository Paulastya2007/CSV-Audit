use cursive::align::HAlign;
use cursive::traits::{Nameable, Resizable, Scrollable};
use cursive::views::{Checkbox, Dialog, DummyView, EditView, LinearLayout, SelectView, TextView};
use cursive::Cursive;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::ui::dialogs;

/// Entry point: shows the CSV file-picker dialog.
/// Layer stack after this: [main_menu, file_picker]
pub fn open_csv_workflow(siv: &mut Cursive) {
    let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    let csv_files: Vec<_> = crate::csv::helpdir::find_csv_files(&root)
        .into_iter()
        .filter_map(|path| crate::csv::backend::inspect_csv(&path).ok())
        .collect();

    let mut select_view = build_csv_select_view(&csv_files);
    let has_files = !csv_files.is_empty();

    // Sync the list selection into the editable path field.
    select_view.set_on_select(|s, path_str| {
        s.call_on_name("csv_path", |view: &mut EditView| {
            view.set_content(path_str);
        });
    });

    let default_path = csv_files
        .first()
        .map(|f| f.path.to_string_lossy().into_owned())
        .unwrap_or_default();

    let path_input = EditView::new()
        .content(default_path)
        .with_name("csv_path")
        .fixed_width(60);

    let mut content = LinearLayout::vertical();

    if has_files {
        content.add_child(TextView::new("Discovered CSV files in directory:"));
        content.add_child(DummyView);
        content.add_child(select_view.scrollable().fixed_height(8));
        content.add_child(DummyView);
    } else {
        content.add_child(TextView::new(
            "No CSV files discovered in root or direct subdirectories.",
        ));
        content.add_child(DummyView);
    }

    content.add_child(TextView::new("Selected File Path:"));
    content.add_child(path_input);

    siv.add_layer(
        Dialog::around(content)
            .title("Open CSV File")
            .button("Open", |s| {
                let file_path = s
                    .call_on_name("csv_path", |view: &mut EditView| view.get_content())
                    .unwrap_or_default();

                if file_path.trim().is_empty() {
                    dialogs::show_error(
                        s,
                        "No file path entered. Please select a file from the list \
                         or type a path manually.",
                    );
                    return;
                }

                let path = Path::new(file_path.as_ref()).to_path_buf();
                // Pop the file-picker so field-selection sits directly above main menu.
                s.pop_layer();
                load_and_select_fields(s, path);
            })
            .button("Cancel", |s| {
                s.pop_layer(); // back to main menu
            }),
    );
}

fn build_csv_select_view(files: &[crate::csv::backend::CsvFile]) -> SelectView<String> {
    let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut select = SelectView::new().h_align(HAlign::Left);
    for file in files {
        select.add_item(file.label(&root), file.path.to_string_lossy().into_owned());
    }
    select
}

/// Asks the user which columns should be editable, then hands off to csv_work.
/// Layer stack: [main_menu, field_selection]
fn load_and_select_fields(siv: &mut Cursive, path: PathBuf) {
    let info = match crate::csv::backend::inspect_csv(&path) {
        Ok(info) => info,
        Err(err) => {
            dialogs::show_error(siv, format!("Failed to open CSV file:\n\n{}", err));
            return;
        }
    };

    let headers = match crate::csv::backend::read_header(&path) {
        Ok(headers) => headers,
        Err(err) => {
            dialogs::show_csv_info(siv, &info, Some(err.as_str()));
            return;
        }
    };

    // Shared state: one bool per column, plus a "none" flag.
    // Must use Arc<Mutex<_>> because Checkbox::on_change requires Send + Sync.
    let none_selected: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    let selected: Arc<Mutex<Vec<bool>>> = Arc::new(Mutex::new(vec![false; headers.len()]));
    let mut fields_layout = LinearLayout::vertical();

    fields_layout.add_child(TextView::new(format!(
        "File : {}\nRows : {}   Size : {} bytes\n\nCheck the columns you want to make editable:",
        info.path.file_name().unwrap_or_default().to_string_lossy(),
        info.row_count,
        info.size_bytes,
    )));
    fields_layout.add_child(DummyView);

    // One checkbox per header column.
    // When a field is checked: clear "none" state AND visually uncheck the None checkbox.
    // When a field is unchecked: no side-effects needed.
    let num_headers = headers.len();
    for (i, header) in headers.iter().enumerate() {
        let selected_clone = selected.clone();
        let none_selected_for_field = none_selected.clone();
        fields_layout.add_child(
            LinearLayout::horizontal()
                .child(
                    Checkbox::new()
                        .on_change(move |s, checked| {
                            selected_clone.lock().unwrap()[i] = checked;
                            if checked {
                                // Deactivate the "None" option — both state and UI.
                                *none_selected_for_field.lock().unwrap() = false;
                                s.call_on_name("none_option", |cb: &mut Checkbox| {
                                    cb.uncheck();
                                });
                            }
                        })
                        .with_name(format!("field_{}", i)),
                )
                .child(TextView::new(format!("  {}", header))),
        );
    }

    fields_layout.add_child(DummyView);

    // "None" option — when ticked: clear all field states AND visually uncheck every field checkbox.
    let none_selected_clone = none_selected.clone();
    let selected_for_none = selected.clone();
    fields_layout.add_child(
        LinearLayout::horizontal()
            .child(
                Checkbox::new()
                    .on_change(move |s, checked| {
                        *none_selected_clone.lock().unwrap() = checked;
                        if checked {
                            // Reset internal state (drop lock before touching UI).
                            {
                                let mut sel = selected_for_none.lock().unwrap();
                                for v in sel.iter_mut() {
                                    *v = false;
                                }
                            }
                            // Visually uncheck every field checkbox.
                            for idx in 0..num_headers {
                                s.call_on_name(&format!("field_{}", idx), |cb: &mut Checkbox| {
                                    cb.uncheck();
                                });
                            }
                        }
                    })
                    .with_name("none_option"),
            )
            .child(TextView::new("  None (nothing editable)")),
    );

    let path_for_next = info.path.clone();

    siv.add_layer(
        Dialog::around(fields_layout.scrollable().fixed_height(12))
            .title("Select Editable Fields")
            .button("Next", move |s| {
                let cols: Vec<usize> = if *none_selected.lock().unwrap() {
                    vec![]
                } else {
                    selected
                        .lock()
                        .unwrap()
                        .iter()
                        .enumerate()
                        .filter_map(|(i, &checked)| if checked { Some(i) } else { None })
                        .collect()
                };
                // Pop field-selection; inspection dialog sits directly above main menu.
                s.pop_layer();
                crate::ui::csv_work::show_inspection_dialog(s, path_for_next.clone(), cols);
            })
            .button("Cancel", |s| {
                s.pop_layer();
            }),
    );
}
