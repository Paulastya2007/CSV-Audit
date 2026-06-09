use cursive::views::{Dialog, TextView};
use cursive::Cursive;

/// Simple message dialog with an Ok button.
pub fn show_message(siv: &mut Cursive, title: &str, message: impl Into<String>) {
    siv.add_layer(
        Dialog::around(TextView::new(message.into()))
            .title(title)
            .button("Ok", |s| {
                s.pop_layer();
            }),
    );
}

/// Simple error dialog.
pub fn show_error(siv: &mut Cursive, message: impl Into<String>) {
    show_message(siv, "Error", message);
}

/// Yes/No confirmation dialog.
pub fn show_confirmation<FYes, FNo>(
    siv: &mut Cursive,
    title: &str,
    message: impl Into<String>,
    yes_label: &str,
    no_label: &str,
    on_yes: FYes,
    on_no: FNo,
) where
    FYes: Fn(&mut Cursive) + Send + Sync + 'static,
    FNo: Fn(&mut Cursive) + Send + Sync + 'static,
{
    siv.add_layer(
        Dialog::around(TextView::new(message.into()))
            .title(title)
            .button(yes_label, move |s| {
                on_yes(s);
            })
            .button(no_label, move |s| {
                on_no(s);
            }),
    );
}
pub fn show_csv_info(siv: &mut Cursive, info: &crate::csv::backend::CsvFile, extra: Option<&str>) {
    let mut msg = format!(
        "File loaded successfully!\n\n\
         Path: {}\n\
         Rows: {}\n\
         Size: {} bytes",
        info.path.display(),
        info.row_count,
        info.size_bytes
    );

    if let Some(err) = extra {
        msg.push_str("\n\nHeader extraction error: ");
        msg.push_str(err);
    }

    show_message(siv, "File Opened", msg);
}
