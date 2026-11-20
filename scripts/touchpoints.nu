#!/usr/bin/env nu
# Every place a design, component or token is wired: the checklist for adding a sibling.
#
#   nu scripts/touchpoints.nu neobrutalism     files to mirror when adding a new design
#   nu scripts/touchpoints.nu DataTable        files to extend when adding a component
#
# Case-insensitive; skips generated output (dist, target, lockfiles, changelog). The count is
# the number of matching lines, so the big rows are where the real work is.

def main [term: string] {
    let rows = (
        ^rg --count-matches --ignore-case --no-messages --glob '!dist/**' --glob '!target/**' --glob '!Cargo.lock' --glob '!CHANGELOG.md' --glob '!docs/PARITY.md' --glob '!scripts/touchpoints.nu' --glob '!**/*.svg' --glob '!**/*.json' $term crates scripts .github docs README.md AGENTS.md
        | lines
        | parse '{file}:{n}'
        | update n { into int }
        | sort-by n --reverse
    )
    let by_crate = ($rows | insert crate { $in.file | split row '/' | skip 1 | first | default 'root' } | group-by crate --to-table | each { |g| { crate: $g.crate, files: ($g.items | length), matches: ($g.items.n | math sum) } } | sort-by matches --reverse)
    print $"(ansi bo)($rows | length) files mention '($term)'(ansi reset)"
    print ($by_crate | table --index false)
    print ($rows | first 40 | table --index false)
}
