# Pure helpers for workspace version handling (unit-tested in scripts/tests).

# Is `v` a semver string (MAJOR.MINOR.PATCH with optional pre-release)?
export def is-semver [v: string]: nothing -> bool {
    $v =~ '^\d+\.\d+\.\d+(-[a-zA-Z0-9.]+)?$'
}

# Read `[workspace.package] version` from Cargo.toml text.
export def read-version [cargo: string]: nothing -> string {
    $cargo | from toml | get workspace.package.version
}

# Rewrite the workspace version and every internal `dodai-*` dependency
# version in a root Cargo.toml text. Other lines are untouched.
export def bump-workspace-text [cargo: string, version: string]: nothing -> string {
    mut in_pkg = false
    mut out = []
    for line in ($cargo | lines) {
        let t = ($line | str trim)
        if ($t | str starts-with '[') { $in_pkg = ($t == '[workspace.package]') }
        let new = if $in_pkg and ($line =~ '^version\s*=') {
            $line | str replace --regex '"[^"]+"' $'"($version)"'
        } else if ($line =~ '^dodai-[a-z-]+\s*=\s*\{') {
            $line | str replace --regex 'version\s*=\s*"[^"]+"' $'version = "($version)"'
        } else { $line }
        $out = ($out | append $new)
    }
    $out | str join "\n"
}

# Crates in dependency order: core must be published before adapters.
export def publish-order []: nothing -> list<string> {
    ["dodai-fx" "dodai-frost" "dodai-core" "dodai-icons" "dodai-shaders" "dodai-ratatui" "dodai-iced" "dodai-gpui" "dodai-topcoat" "dodai-dioxus" "dodai-bevy"]
}
