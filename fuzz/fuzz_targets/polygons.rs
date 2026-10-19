#![no_main]
use libfuzzer_sys::fuzz_target;

// Geometry helpers must cope with any points (NaN, infinities, duplicates, tiny rings).
fuzz_target!(|data: (Vec<(f32, f32)>, f32)| {
    let (pts, tol) = data;
    let _ = dodai_material_icons::octilinear(&pts);
    let _ = dodai_material_icons::simplify_closed(&pts, tol);
    let _ = dodai_material_icons::facet_polygon(&pts);
});
