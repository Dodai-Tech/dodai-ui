#![no_main]
use libfuzzer_sys::fuzz_target;

// The icon packs' internal name tables are keyed by text: whatever is fed in must never panic.
fuzz_target!(|name: &str| {
    let _ = dodai_cyberpunk_icons::split_modifier(name);
    let _ = dodai_cyberpunk_icons::lookup(name);
    let _ = dodai_material_icons::alias_of(name);
    let _ = dodai_material_icons::find(dodai_material_icons::outlined::ALL, name);
    let _ = dodai::icons::IconId::from_key(name);
});
