//! Prints `theme.slint` for the Cyberpunk default theme (or the theme id given as arg).
fn main() {
    let t = std::env::args()
        .nth(1)
        .and_then(|id| dodai_ui_core::themes::by_id(&id))
        .unwrap_or_else(dodai_ui_core::themes::cyberpunk::yellow);
    print!("{}", dodai_ui_slint::generate_theme_slint(&t));
}
