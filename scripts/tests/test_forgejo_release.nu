#!/usr/bin/env nu
# ── Dodai UI · test_forgejo_release.nu ──────────────────────────────────────
# Tests for scripts/ci/forgejo_release.nu — release payload.

use std/assert
use runner.nu *
use ../ci/forgejo_release.nu [release_payload]

def "test forgejo_release: payload carries tag, title and notes" [] {
    let p = (release_payload "v1.2.3" "notes")
    assert equal $p.tag_name "v1.2.3"
    assert equal $p.name "Dodai UI v1.2.3"
    assert equal $p.body "notes"
}

# ── Main ────────────────────────────────────────────────────────────────────

def main [] { run-tests }
