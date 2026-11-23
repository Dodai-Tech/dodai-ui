#![no_main]
use libfuzzer_sys::fuzz_target;

// The icon packs' internal name tables are keyed by text: whatever is fed in must never panic.
fuzz_target!(|name: &str| {
    let _ = dodai_icons::cyberpunk::split_modifier(name);
    let _ = dodai_icons::cyberpunk::lookup(name);
    let _ = dodai_icons::material::alias_of(name);
    let _ = dodai_icons::material::find(dodai_icons::material::outlined::ALL, name);
    let _ = dodai_icons::IconId::from_key(name);
});
