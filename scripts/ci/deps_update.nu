#!/usr/bin/env nu
# ── Dodai UI · nightly dependency update helpers (used by deps-update.yml) ───
#   nu scripts/ci/deps_update.nu upgrade             cargo upgrade (compatible requirements) + cargo update
#   nu scripts/ci/deps_update.nu changed             prints changed=… and release=… (→ $GITHUB_OUTPUT)
#   nu scripts/ci/deps_update.nu release <bot> <mail> commit, bump the patch (changelog, tag), push, release
#
# The dodai crates are libraries: their users see the requirements in the manifests, never the
# lockfile. So a raised requirement ships as a patch release; a lockfile-only change is not
# released. Majors are never taken here (they are breaking: do them by hand). crates/dodai-freya
# is its own workspace, pinned to one freya release; it is not touched.
# ──────────────────────────────────────────────────────────────────────────────

use ../lib/versioning.nu [read-version]

# Versions held back on purpose: `cargo update` may not go past `max`. Drop a row when the reason is gone.
#   libc > 0.2.189 removed ENOATTR on linux-gnu, which xattr 0.2.3 (and 1.7.0) still use.
#   mio 1.2.5 needs libc >= 0.2.190, so it is held too.
const HOLDS = [
    {name: "mio", max: "1.2.4"}
    {name: "libc", max: "0.2.189"}
    {name: "xattr", max: "1.6.1"}
]

const KINDS = ["dependencies" "dev-dependencies" "build-dependencies"]

# The next patch version: 0.1.4 → 0.1.5 (a pre-release ships as its release: 1.0.0-rc.1 → 1.0.0).
export def next_patch [version: string]: nothing -> string {
    let core = ($version | split row "-" | first | split row "." | each { into int })
    if ($version | str contains "-") { $core | str join "." } else { [$core.0 $core.1 ($core.2 + 1)] | str join "." }
}

export def upgrade_message [date: string]: nothing -> string {
    $"chore\(deps\): upgrade dependencies ($date)"
}

# The `cargo update -p <name>@<version> --precise <max>` calls that put a lockfile back under the
# held versions. `lock` is the parsed Cargo.lock; versions compare as dotted numbers.
export def held_back [lock: record, holds: list<record>]: nothing -> list<record> {
    let num = {|v| $v | split row "." | each { into int } }
    $holds | each {|h|
        $lock.package | where name == $h.name | each {|p|
            if (do $num $p.version) > (do $num $h.max) { {spec: $"($h.name)@($p.version)", to: $h.max} }
        }
    } | flatten | compact
}

# Every dependency requirement of a parsed Cargo.toml, as `kind name` → version (target tables and
# `workspace.dependencies` included). Formatting and key order do not matter.
export def pins [cargo: record]: nothing -> record {
    let tables = ($KINDS | each {|k| {prefix: $k, deps: ($cargo | get -o $k | default {})} })
        | append {prefix: "workspace", deps: ($cargo | get -o workspace.dependencies | default {})}
        | append ($cargo | get -o target | default {} | transpose cfg t | each {|row|
            $KINDS | each {|k| {prefix: $"($row.cfg) ($k)", deps: ($row.t | get -o $k | default {})} }
        } | flatten)
    $tables | each {|t|
        $t.deps | transpose name spec | each {|d|
            let v = if ($d.spec | describe) == "string" { $d.spec } else { $d.spec | get -o version | default "" }
            {key: $"($t.prefix) ($d.name)", value: $v}
        }
    } | flatten | reduce --fold {} {|it, acc| $acc | insert $it.key $it.value }
}

# Did a dependency requirement change between two parsed Cargo.toml files?
export def pins_changed [before: record, after: record]: nothing -> bool {
    (pins $before) != (pins $after)
}

# The workspace's manifests, relative: the root and every member.
def manifests []: nothing -> list<string> {
    let m = ^cargo metadata --no-deps --format-version 1 | from json
    $m.packages.manifest_path | each {|p| $p | path relative-to $m.workspace_root } | prepend Cargo.toml | uniq
}

def committed [manifest: string] { ^git show $"HEAD:($manifest)" | from toml }

def "main upgrade" [] {
    ^cargo upgrade
    ^cargo update
    for h in (held_back (open --raw Cargo.lock | from toml) $HOLDS) { ^cargo update -p $h.spec --precise $h.to }
}

def "main changed" [] {
    let lock = do { ^git diff --quiet -- Cargo.lock } | complete
    let release = (manifests | any {|m| pins_changed (committed $m) (open $m) })
    # stdout is $GITHUB_OUTPUT: key=value lines only; the human part goes to stderr.
    if $release or $lock.exit_code != 0 { print -e (^git diff --stat) }
    print $"changed=($release or $lock.exit_code != 0)"
    print $"release=($release)"
}

def "main commit" [bot: string, email: string] {
    ^git config user.name $bot
    ^git config user.email $email
    ^git add Cargo.lock
    ^git commit -m $"chore\(deps\): nightly cargo update (date now | date to-timezone UTC | format date '%Y-%m-%d')"
    ^git push origin HEAD:main
}

# Commit the upgraded requirements, bump the patch (changelog, tag) and push both. The tag push
# starts release.yml only when it was made with a PAT; a GITHUB_TOKEN push starts no workflow.
def "main release" [bot: string, email: string] {
    ^git config user.name $bot
    ^git config user.email $email
    ^git add ...(manifests) Cargo.lock
    ^git commit -m (upgrade_message (date now | date to-timezone UTC | format date "%Y-%m-%d"))
    let next = next_patch (read-version (open Cargo.toml --raw))
    ^nu scripts/bump_version.nu $next --skip-checks
    ^git push --follow-tags origin HEAD:main
    if ($env.HAS_PAT? | default "false") != "true" {
        ^gh workflow run release.yml --ref main -f $"tag=v($next)"
    }
    print $"released=v($next)"
}

def main [] { print "usage: nu scripts/ci/deps_update.nu (upgrade | changed | commit <bot> <email> | release <bot> <email>)" }
