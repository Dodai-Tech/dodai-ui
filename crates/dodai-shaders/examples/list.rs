use dodai_shaders::catalog;
fn main() {
    for s in catalog() {
        println!(
            "{}|{}|{:?}|{}",
            if s.is_generator() { "G" } else { "E" },
            s.id,
            s.family,
            s.desc
        );
    }
}
