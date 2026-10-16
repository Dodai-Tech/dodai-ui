#!/usr/bin/env nu
# Pre-publish checks: docs build, `cargo publish --dry-run` for the core crate
# (adapters can only be dry-run once core is on the registry), cargo check for all.
# Usage: nu scripts/check_publish.nu

use lib/versioning.nu *

def run [label: string, cmd: closure] {
    print $"  ($label)…"
    let r = (do $cmd | complete)
    if $r.exit_code != 0 {
        print $"  (ansi red)❌ ($label) failed(ansi reset)"
        print $r.stderr
        exit 1
    }
    print $"  (ansi green)✅ ($label)(ansi reset)"
}

def main [] {
    print "── Legendary UI · pre-publish checks ──"
    for c in (publish-order) {
        run $"doc ($c)" { cargo doc -p $c --no-deps }
    }
    run "publish dry-run legendary-core" { cargo publish --dry-run --allow-dirty -p legendary-core }
    for c in (publish-order) {
        run $"check ($c)" { cargo check -p $c }
    }
    print $"(ansi green)All pre-publish checks passed.(ansi reset)"
}
