#![no_main]
use libfuzzer_sys::fuzz_target;

// The Material path parser takes arbitrary text; it must never panic, and
// whatever it returns must survive the Cyberpunk faceting.
fuzz_target!(|d: &str| {
    for poly in dodai_material_icons::flatten(d) {
        let _ = dodai_material_icons::facet_polygon(&poly);
    }
});
