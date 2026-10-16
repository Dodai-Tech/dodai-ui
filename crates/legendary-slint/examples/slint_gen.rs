//! Prints `theme.slint` for the default theme (or the theme id given as arg).
fn main() {
    let t = std::env::args()
        .nth(1)
        .and_then(|id| legendary_core::themes::by_id(&id))
        .unwrap_or_default();
    print!("{}", legendary_slint::generate_theme_slint(&t));
}
