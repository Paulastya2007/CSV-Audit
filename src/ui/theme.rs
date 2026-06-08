use cursive::theme::{BorderStyle, Color, PaletteColor, Theme};
use cursive::Cursive;

/*
 * Set up the modern dark palette theme with shadows enabled.
 */
pub fn apply_theme(siv: &mut Cursive) {
    let mut theme = Theme {
        borders: BorderStyle::Outset,
        shadow: true,
        ..Theme::default()
    };

    theme.palette[PaletteColor::Background] = Color::Rgb(24, 26, 32);
    theme.palette[PaletteColor::View] = Color::Rgb(33, 37, 43);
    theme.palette[PaletteColor::Primary] = Color::Rgb(220, 223, 228);
    theme.palette[PaletteColor::Secondary] = Color::Rgb(157, 165, 180);
    theme.palette[PaletteColor::TitlePrimary] = Color::Rgb(97, 175, 239);
    theme.palette[PaletteColor::TitleSecondary] = Color::Rgb(198, 120, 221);
    theme.palette[PaletteColor::Highlight] = Color::Rgb(97, 175, 239);
    theme.palette[PaletteColor::HighlightText] = Color::Rgb(24, 26, 32);
    theme.palette[PaletteColor::HighlightInactive] = Color::Rgb(75, 82, 99);

    siv.set_theme(theme);
}
