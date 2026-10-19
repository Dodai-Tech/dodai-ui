#![no_main]
use libfuzzer_sys::fuzz_target;

// Theme ids and icon glyphs resolved for every built-in theme.
fuzz_target!(|data: (&str, bool)| {
    let (name, filled) = data;
    let _ = dodai::themes::by_id(name);
    for t in dodai::themes::all() {
        let _ = dodai::icons::glyph(name, &t, filled);
    }
});
