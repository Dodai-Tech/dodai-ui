//! Prints `theme.slint` for the default theme (or the theme id given as arg).
fn main() {
    let t = std::env::args()
        .nth(1)
        .and_then(|id| dodai_core::themes::by_id(&id))
        .unwrap_or_default();
    print!("{}", dodai_slint::generate_theme_slint(&t));
}
