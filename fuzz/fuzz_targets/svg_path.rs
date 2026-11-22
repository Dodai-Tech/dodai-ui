#![no_main]
use libfuzzer_sys::fuzz_target;

// The Material path parser takes arbitrary text; it must never panic, and
// whatever it returns must survive the Cyberpunk faceting.
fuzz_target!(|d: &str| {
    for poly in dodai_ui_icons::material::flatten(d) {
        let _ = dodai_ui_icons::material::facet_polygon(&poly);
    }
});
