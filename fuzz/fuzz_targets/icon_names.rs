#![no_main]
use libfuzzer_sys::fuzz_target;

// Icon names come from app code and user input: lookups must never panic.
fuzz_target!(|name: &str| {
    let _ = dodai_cyberpunk_icons::split_modifier(name);
    let _ = dodai_cyberpunk_icons::lookup(name);
    let _ = dodai_material_icons::resolve(name);
    let _ = dodai_material_icons::alias_of(name);
    let _ = dodai_material_icons::find(dodai_material_icons::outlined::ALL, name);
});
