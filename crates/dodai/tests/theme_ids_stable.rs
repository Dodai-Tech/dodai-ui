//! Building the built-in themes again must not allocate new ids (the fuzz target calls
//! `themes::all()` every iteration; a leak per call failed LeakSanitizer in CI).

#[test]
fn rebuilding_the_themes_reuses_the_id_strings() {
    let first: Vec<&'static str> = dodai::themes::all().iter().map(|t| t.id()).collect();
    let second: Vec<&'static str> = dodai::themes::all().iter().map(|t| t.id()).collect();
    assert_eq!(first.len(), second.len());
    for (a, b) in first.iter().zip(&second) {
        assert!(std::ptr::eq(*a, *b), "`{a}` was allocated again");
    }
}
