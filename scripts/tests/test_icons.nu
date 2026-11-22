#!/usr/bin/env nu
# ── Dodai UI · test_icons.nu ────────────────────────────────────────────────
# Tests for scripts/icons.nu — app icon generation helpers.

use std/assert
use runner.nu *
use ../icons.nu [u32-be icns-entry icns-bytes detect-id gui-assets-dir outputs raster-args pick-rasteriser]

def "test icons: u32-be encodes big-endian" [] {
    assert equal (u32-be 258) 0x[00 00 01 02]
    assert equal (u32-be 0) 0x[00 00 00 00]
}

def "test icons: icns entry carries type and length" [] {
    let e = icns-entry "ic07" 0x[AA BB]
    assert equal $e 0x[69 63 30 37 00 00 00 0A AA BB]
}

def "test icons: icns file wraps entries" [] {
    let f = icns-bytes [0x[01 02]]
    assert equal ($f | bytes at 0..3) ("icns" | into binary)
    assert equal ($f | bytes length) 10
}

def "test icons: id from a lone master" [] {
    assert equal (detect-id ["assets/icons/dodai-ui.svg"]) "dodai-ui"
    assert equal (detect-id []) null
    assert equal (detect-id ["a.svg" "b.svg"]) null
}

def "test icons: gui assets dir falls back" [] {
    assert equal (gui-assets-dir []) "assets/app"
    assert equal (gui-assets-dir ["crates/z-gui/assets" "crates/a-gui/assets"]) "crates/a-gui/assets"
}

def "test icons: outputs name every asset" [] {
    let out = outputs "dodai-ui" "crates/dodai-ui-showcase/assets"
    assert ("crates/dodai-ui-showcase/assets/dodai-ui-macos-512.png" in $out)
    assert ("packaging/macos/dodai-ui.icns" in $out)
    assert ("assets/icons/dodai-ui-16.png" in $out)
}

def "test icons: rasteriser argv" [] {
    assert equal (raster-args "resvg" "a.svg" 32 "o.png") ["--width" "32" "--height" "32" "a.svg" "o.png"]
    assert equal (pick-rasteriser ["rsvg-convert"]) "rsvg-convert"
    assert equal (pick-rasteriser []) null
}

# ── Main ────────────────────────────────────────────────────────────────────

def main [] { run-tests }
