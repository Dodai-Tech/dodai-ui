#![no_main]
use libfuzzer_sys::fuzz_target;

// Theme ids by string, and any shared-vocabulary icon resolved for every built-in theme.
fuzz_target!(|data: (&str, u16, bool)| {
    let (name, idx, filled) = data;
    let _ = dodai::themes::by_id(name);
    let id = dodai::icons::IconId::ALL[idx as usize % dodai::icons::IconId::ALL.len()];
    for t in dodai::themes::all() {
        let _ = dodai::icons::glyph(id, &t, filled);
    }
});
