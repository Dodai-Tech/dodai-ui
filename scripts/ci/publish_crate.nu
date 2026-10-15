#!/usr/bin/env nu
# Publish one crate to crates.io, skipping if that exact version already exists.
# Usage: nu scripts/ci/publish_crate.nu <crate>      (needs CARGO_REGISTRY_TOKEN)

def main [crate: string] {
    let version = (open Cargo.toml | get workspace.package.version)
    let url = $"https://crates.io/api/v1/crates/($crate)/($version)"
    let exists = (try { http get --full --allow-errors $url | get status } catch { 0 }) == 200
    if $exists {
        print $"($crate) ($version) already published — skipping."
        return
    }
    run-external "cargo" "publish" "-p" $crate "--locked"
}
