//! Prints `kind id` for every catalogue entry (used by the kernel generator).
fn main() {
    for s in dodai_ui_shaders::catalog() {
        println!("{} {}", if s.is_generator() { "gen" } else { "post" }, s.id);
    }
}
