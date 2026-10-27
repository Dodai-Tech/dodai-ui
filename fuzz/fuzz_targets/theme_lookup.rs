#![no_main]
use libfuzzer_sys::fuzz_target;

// Theme ids by string, and any shared-vocabulary icon resolved for every built-in theme.
fuzz_target!(|data: (&str, u16, bool)| {
    let (name, idx, filled) = data;
    let _ = dodai_core::themes::by_id(name);
    let id = dodai_icons::IconId::ALL[idx as usize % dodai_icons::IconId::ALL.len()];
    for t in dodai_core::themes::all() {
        let _ = dodai_icons::glyph_for_theme(id, &t, filled);
    }
});
