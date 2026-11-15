#!/usr/bin/env nu
# ──────────────────────────────────────────────────────────────────────────────
# Dodai UI — Bump workspace version
# ──────────────────────────────────────────────────────────────────────────────
# Usage: nu scripts/bump_version.nu <new_version>
#
#   1. Validates semver.
#   2. Updates workspace.package.version + internal dodai-* dep versions.
#   3. fmt, clippy, test (--skip-checks: fmt only, for a run that gated already).
#   4. Refreshes Cargo.lock and regenerates CHANGELOG.md via git-cliff.
#   5. Commits and tags vX.Y.Z.
# ──────────────────────────────────────────────────────────────────────────────

use lib/versioning.nu *

def main [new_version: string, --skip-checks] {
    if not (is-semver $new_version) {
        print $"(ansi red)Error:(ansi reset) '($new_version)' is not a valid semantic version."
        exit 1
    }
    let current = (read-version (open Cargo.toml --raw))
    print $"Current: (ansi yellow)($current)(ansi reset)  →  New: (ansi green)($new_version)(ansi reset)"
    if $current == $new_version {
        print $"(ansi yellow)⚠(ansi reset) Already at ($new_version). Nothing to do."
        exit 0
    }

    bump-workspace-text (open Cargo.toml --raw) $new_version | $in + "\n" | save --force Cargo.toml
    print $"(ansi green)✓(ansi reset) Cargo.toml updated."

    run-external "cargo" "fmt" "--all"
    if not $skip_checks {
        run-external "cargo" "clippy" "--workspace" "--all-targets" "--" "-D" "warnings"
        run-external "cargo" "test" "--workspace"
    }
    run-external "cargo" "update" "--workspace"

    let written = (open --raw CHANGELOG.md | str contains $"## [($new_version)]")
    if $written {
        print $"(ansi green)✓(ansi reset) CHANGELOG.md already has a ($new_version) section, kept."
    } else if (which git-cliff | is-not-empty) {
        run-external "git-cliff" "--output" "CHANGELOG.md" "--tag" $"v($new_version)"
        print $"(ansi green)✓(ansi reset) CHANGELOG.md updated."
    } else {
        print $"(ansi yellow)⚠(ansi reset) git-cliff not found — skipping changelog."
    }

    run-external "git" "add" "-A"
    run-external "git" "commit" "-m" $"chore: bump version to ($new_version)"
    run-external "git" "tag" "-a" $"v($new_version)" "-m" $"Release v($new_version)"
    print $"(ansi green)✓(ansi reset) Tagged v($new_version). Next: git push origin main --tags"
}
