#!/usr/bin/env nu
# Write release notes for the tag being released to RELEASE_NOTES.md (git-cliff).
# Usage: nu scripts/ci/release_notes.nu <tag>

use ../lib/versioning.nu changelog-section

def main [tag: string] {
    # A hand-written CHANGELOG.md section for this version is the release notes.
    let written = if ("CHANGELOG.md" | path exists) {
        changelog-section (open --raw CHANGELOG.md) ($tag | str trim --left --char "v")
    } else { "" }
    if $written != "" {
        $written | save --force RELEASE_NOTES.md
        print $"Wrote RELEASE_NOTES.md for ($tag) from CHANGELOG.md"
        return
    }
    if (which git-cliff | is-empty) {
        print --stderr "git-cliff not found"; exit 1
    }
    run-external "git-cliff" "--latest" "--strip" "header" "--tag" $tag "--output" "RELEASE_NOTES.md"
    print $"Wrote RELEASE_NOTES.md for ($tag)"
}
