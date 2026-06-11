use crate::csv::session::Session;
use crate::ui::csv_work;
use crate::ui::csv_workflow;
use crate::ui::dialogs;
use cursive::align::HAlign;
use cursive::views::{Dialog, DummyView, LinearLayout, SelectView, TextView};
use cursive::Cursive;

#[derive(Clone, Copy, Debug, PartialEq)]
enum MenuOption {
    OpenCsv,
    RestoreSession,
    Exit,
}

pub fn show_main_menu(siv: &mut Cursive) {
    let mut menu_select = SelectView::<MenuOption>::new()
        .h_align(HAlign::Center)
        .autojump();

    menu_select.add_item("1. Open CSV File", MenuOption::OpenCsv);
    menu_select.add_item("2. Restore Previous Session", MenuOption::RestoreSession);
    menu_select.add_item("3. Exit Application", MenuOption::Exit);

    menu_select.set_on_submit(|s, option| match option {
        MenuOption::OpenCsv => csv_workflow::open_csv_workflow(s),
        MenuOption::RestoreSession => restore_session_workflow(s),
        MenuOption::Exit => confirm_exit_workflow(s),
    });

    let layout = LinearLayout::vertical()
        .child(TextView::new("🔒 PASSWORD SANITIZATION TOOL 🔒").h_align(HAlign::Center))
        .child(DummyView)
        .child(TextView::new("Please choose an operation:").h_align(HAlign::Center))
        .child(DummyView)
        .child(menu_select)
        .child(DummyView)
        .child(
            TextView::new(format!(
                "Use Arrow Keys/Numbers to navigate. Press Enter to select.\nVersion {}",
                env!("CARGO_PKG_VERSION")
            ))
            .h_align(HAlign::Center),
        );

    siv.add_layer(
        Dialog::around(layout)
            .title("Main Menu")
            .padding_lrtb(4, 4, 2, 2),
    );
}

fn restore_session_workflow(siv: &mut Cursive) {
    /* Scan current directory for .session.json files */
    let root = std::env::current_dir().unwrap_or_default();
    let sessions = find_session_files(&root);

    if sessions.is_empty() {
        dialogs::show_message(
            siv,
            "No Sessions Found",
            "No saved session files were found in the current directory.",
        );
        return;
    }

    if sessions.len() == 1 {
        /* Only one session — show details and ask to restore */
        let session = sessions.into_iter().next().unwrap();
        show_restore_prompt(siv, session);
        return;
    }

    /* Multiple sessions — let user pick */
    let mut select = SelectView::<Session>::new().h_align(HAlign::Left);

    for sess in sessions {
        let label = format!(
            "{} — record {}/{} — {}",
            sess.original_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy(),
            sess.current_index + 1,
            sess.total_rows,
            sess.last_activity,
        );
        select.add_item(label, sess);
    }

    select.set_on_submit(|s, session| {
        s.pop_layer();
        show_restore_prompt(s, session.clone());
    });

    siv.add_layer(
        Dialog::around(select)
            .title("Select Session to Restore")
            .button("Cancel", |s| {
                s.pop_layer();
            }),
    );
}

fn show_restore_prompt(siv: &mut Cursive, session: Session) {
    let msg = format!(
        "Found saved session:\n\n\
         Original file: {}\n\
         Output file:   {}\n\
         Progress:      record {} of {}\n\
         Saved:         {} records\n\
         Discarded:     {} records\n\
         Started:       {}\n\
         Last activity: {}\n\n\
         Would you like to restore this session?",
        session.original_path.display(),
        session.output_path.display(),
        session.current_index + 1,
        session.total_rows,
        session.saved_count,
        session.discarded_count,
        session.started_at,
        session.last_activity,
    );

    let session_restore = session.clone();
    let session_delete = session.clone();

    siv.add_layer(
        Dialog::around(TextView::new(msg))
            .title("Restore Previous Session")
            .button("Yes, Restore", move |s| {
                s.pop_layer();
                csv_work::restore_session(s, session_restore.clone());
            })
            .button("No, Delete Session", move |s| {
                s.pop_layer();
                let _ = Session::delete(&session_delete.original_path);
                dialogs::show_message(s, "Deleted", "Session file has been removed.");
            })
            .button("Cancel", |s| {
                s.pop_layer();
            }),
    );
}

/// Find all `.session.json` files in the given directory.
fn find_session_files(root: &std::path::Path) -> Vec<Session> {
    let mut sessions = Vec::new();

    let entries = match std::fs::read_dir(root) {
        Ok(e) => e,
        Err(_) => return sessions,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file()
            && let Some(name) = path.file_name()
        {
            let name_str = name.to_string_lossy();

            if name_str.ends_with(".session.json") {
                let original = name_str.trim_end_matches(".session.json");
                let original_path = root.join(original);

                if let Ok(Some(session)) = Session::load(&original_path) {
                    sessions.push(session);
                }
            }
        }
    }

    sessions
}

fn confirm_exit_workflow(siv: &mut Cursive) {
    dialogs::show_confirmation(
        siv,
        "Exit Confirmation",
        "Are you sure you want to exit the application?",
        "Yes, Exit",
        "No, Stay",
        |s| s.quit(),
        |s| {
            s.pop_layer();
        },
    );
}
