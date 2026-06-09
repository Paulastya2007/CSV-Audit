use software_rust::ui;

fn main() {
    let mut siv = cursive::default();

    ui::theme::apply_theme(&mut siv);
    ui::menu::show_main_menu(&mut siv);

    siv.run();
}
