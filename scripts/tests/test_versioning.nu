#!/usr/bin/env nu
# Tests for scripts/lib/versioning.nu

use std/assert
use runner.nu *
use ../lib/versioning.nu *

def sample [v: string]: nothing -> string {
    $'[workspace]
members = ["crates/dodai-core"]

[workspace.package]
version      = "($v)"
edition      = "2021"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
dodai-core    = { path = "crates/dodai-core",    version = "($v)" }
dodai-ratatui = { path = "crates/dodai-ratatui", version = "($v)" }
'
}

def "test semver: accepts plain and pre-release" [] {
    assert (is-semver "0.1.0")
    assert (is-semver "12.34.56")
    assert (is-semver "1.0.0-rc.1")
}

def "test semver: rejects junk" [] {
    assert (not (is-semver "1.0"))
    assert (not (is-semver "v1.0.0"))
    assert (not (is-semver "1.0.0."))
    assert (not (is-semver ""))
}

def "test read-version: reads workspace.package" [] {
    assert equal (read-version (sample "0.3.1")) "0.3.1"
}

def "test bump: updates workspace + internal deps" [] {
    let out = (bump-workspace-text (sample "0.1.0") "0.2.0")
    assert equal (read-version $out) "0.2.0"
    assert ($out | str contains 'dodai-core    = { path = "crates/dodai-core",    version = "0.2.0" }')
    assert ($out | str contains 'dodai-ratatui = { path = "crates/dodai-ratatui", version = "0.2.0" }')
}

def "test bump: leaves third-party deps alone" [] {
    let out = (bump-workspace-text (sample "0.1.0") "9.9.9")
    assert ($out | str contains 'serde = { version = "1", features = ["derive"] }')
}

def "test bump: is idempotent" [] {
    let once = (bump-workspace-text (sample "0.1.0") "0.2.0")
    assert equal (bump-workspace-text $once "0.2.0") $once
}

def "test publish-order: leaves first, no duplicates, leaves after their deps" [] {
    let o = (publish-order)
    assert equal ($o | first) "dodai-motion"
    assert (($o | enumerate | where item == "dodai-motion" | get index.0) < ($o | enumerate | where item == "dodai-core" | get index.0))
    assert equal ($o | uniq | length) ($o | length)
    # the umbrella and the adapters come after the design systems they depend on
    assert (($o | enumerate | where item == "dodai-cyberpunk" | get index.0) < ($o | enumerate | where item == "dodai" | get index.0))
    assert (($o | enumerate | where item == "dodai-material" | get index.0) < ($o | enumerate | where item == "dodai" | get index.0))
    assert (($o | enumerate | where item == "dodai" | get index.0) < ($o | enumerate | where item == "dodai-gpui" | get index.0))
}

def "test real Cargo.toml: version is semver and bump round-trips" [] {
    let raw = (open ($env.FILE_PWD | path join ".." ".." "Cargo.toml") --raw)
    let v = (read-version $raw)
    assert (is-semver $v)
    assert equal (read-version (bump-workspace-text $raw "7.7.7")) "7.7.7"
}

def main [] { run-tests }
