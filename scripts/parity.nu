#!/usr/bin/env nu
# Generate docs/PARITY.md: what every toolkit covers, measured from source (nothing hand-kept).
#
#   nu scripts/parity.nu           rewrite docs/PARITY.md
#   nu scripts/parity.nu --check   fail if docs/PARITY.md is stale (CI) or a model renderer drops a Node
#
# Three measurements:
#   1. Model coverage: every `Node` variant of dodai-showcase-model against each toolkit that
#      walks the model (a renderer that stops matching a variant is a regression).
#   2. Component coverage: every public gpui component (the reference) against each toolkit's
#      public items (`pub struct|enum|fn`, Slint `component`, topcoat `.dodai-*` CSS classes).
#   3. Token use: whether the toolkit reads Sizes / Look / navigation tokens / the shape layer.

use lib/parity.nu *

def main [--check] {
    let md = (render-parity)
    let out = "docs/PARITY.md"
    if $check {
        let missing = (model-gaps)
        if ($missing | is-not-empty) {
            error make { msg: $"model renderers drop Node variants:\n($missing | to text)" }
        }
        if (not ($out | path exists)) or ((open --raw $out) != $md) {
            error make { msg: "docs/PARITY.md is stale: run `nu scripts/parity.nu` and commit it" }
        }
        print "parity: up to date"
    } else {
        $md | save --force $out
        print $"wrote ($out)"
    }
}
