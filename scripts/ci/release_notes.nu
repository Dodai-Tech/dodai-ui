#!/usr/bin/env nu
# Write release notes for the tag being released to RELEASE_NOTES.md (git-cliff).
# Usage: nu scripts/ci/release_notes.nu <tag>

def main [tag: string] {
    if (which git-cliff | is-empty) {
        print --stderr "git-cliff not found"; exit 1
    }
    run-external "git-cliff" "--latest" "--strip" "header" "--tag" $tag "--output" "RELEASE_NOTES.md"
    print $"Wrote RELEASE_NOTES.md for ($tag)"
}
