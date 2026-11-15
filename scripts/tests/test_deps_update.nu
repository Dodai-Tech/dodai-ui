#!/usr/bin/env nu
# ── Dodai UI · test_deps_update.nu ──────────────────────────────────────────
# Tests for scripts/ci/deps_update.nu — patch version and requirement changes.

use std/assert
use runner.nu *
use ../ci/deps_update.nu [next_patch pins pins_changed upgrade_message]

def "test deps_update: the patch goes up" [] {
    assert equal (next_patch "0.1.4") "0.1.5"
    assert equal (next_patch "1.9.9") "1.9.10"
}

def "test deps_update: a pre-release ships as its release" [] {
    assert equal (next_patch "1.0.0-rc.1") "1.0.0"
}

def "test deps_update: a raised requirement is a change, a reordered file is not" [] {
    let a = ('[dependencies]
tokio = "1.53"
serde = { version = "1", features = ["derive"] }' | from toml)
    let same = ('[dependencies]
serde = { features = ["derive"], version = "1" }
tokio = "1.53"' | from toml)
    let raised = ('[dependencies]
tokio = "1.54"
serde = { version = "1", features = ["derive"] }' | from toml)
    assert (not (pins_changed $a $same))
    assert (pins_changed $a $raised)
}

def "test deps_update: workspace and target tables count" [] {
    let a = {workspace: {dependencies: {wgpu: "30.0.1"}}, target: {"cfg(unix)": {dependencies: {libc: "0.2"}}}}
    let p = (pins $a)
    assert equal ($p | get "workspace wgpu") "30.0.1"
    assert equal ($p | get "cfg(unix) dependencies libc") "0.2"
}

def "test deps_update: the commit message is conventional" [] {
    assert equal (upgrade_message "2026-11-16") "chore(deps): upgrade dependencies 2026-11-16"
}

def main [] { run-tests }
