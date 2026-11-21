#!/usr/bin/env nu
# Regenerates the per-toolkit `generated.rs` from spec/components.toml.
# `--check` fails when a generated file is out of date (CI).

use lib/components.nu render

def main [--check] {
    let root = ($env.FILE_PWD | path dirname)
    let spec = (open ($root | path join spec components.toml))
    mut stale = []
    for t in ($spec.target | transpose name target) {
        let path = ($root | path join $t.target.path)
        let text = (render $spec $t.target | rustfmt --edition 2021 --emit stdout | lines | skip 0 | str join "\n")
        let text = $"($text)\n"
        if $check {
            if (not ($path | path exists)) or ((open --raw $path) != $text) { $stale = ($stale | append $t.target.path) }
        } else {
            $text | save --force $path
            print $"wrote ($t.target.path)"
        }
    }
    if $check and ($stale | is-not-empty) {
        error make --unspanned { msg: $"out of date, run nu scripts/gen-components.nu: ($stale | str join ', ')" }
    }
}
