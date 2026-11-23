# dodai-ui workspace — task runner
# Install just:      cargo install just
# Install git-cliff: cargo install git-cliff
# Usage: just <task>

default:
    @just --list

# ── Tool checks ───────────────────────────────────────────────────────────────

_check-git-cliff:
    @command -v git-cliff >/dev/null 2>&1 || { echo "❌ git-cliff not found. Install with: cargo install git-cliff"; exit 1; }

_check-nu:
    @command -v nu >/dev/null 2>&1 || { echo "❌ nu (nushell) not found. Install: https://www.nushell.sh"; exit 1; }

# Install all recommended development tools
install-tools:
    @command -v git-cliff >/dev/null 2>&1 || cargo install git-cliff --locked
    @command -v nu >/dev/null 2>&1 || cargo install nu --locked
    @command -v cargo-deny >/dev/null 2>&1 || cargo install cargo-deny --locked
    @command -v cargo-audit >/dev/null 2>&1 || cargo install cargo-audit --locked
    @command -v cargo-machete >/dev/null 2>&1 || cargo install cargo-machete --locked
    @echo "✅ All tools installed!"

# ── Build ─────────────────────────────────────────────────────────────────────

# Build the whole workspace (dev). Per-crate: `cargo build --workspace` cannot unify
# iced's naga 27 with gpui's naga 25 (see dodai-iced dev-dependencies).
build:
    for c in dodai-core dodai-charts dodai-ratatui dodai-iced dodai-gpui dodai-topcoat; do cargo build -p $c || exit 1; done

build-core:
    cargo build -p dodai-core

build-shaders:
    cargo build -p dodai-shaders

# Regenerate the GPU kernels (rust-gpu → WGSL). First run installs the pinned
# nightly and builds the codegen backend (~5 min): `cargo install cargo-gpu`.
build-kernels:
    nu scripts/gen-kernels.nu
    cd tools/shaders-kernels && cargo gpu build --shader-crate . --output-dir ../../target/kernels --auto-install-rust-toolchain --target spirv-unknown-naga-wgsl
    cp target/kernels/dodai_shaders_kernels.wgsl crates/dodai-shaders/kernels.wgsl

# Render every shader to PPM files (GPU): just dump-shaders /tmp/dump 2.0
dump-shaders dir="shader-dump" t="1.5":
    cargo run --release -p dodai-shaders --example dump -- {{dir}} {{t}}

build-charts:
    cargo build -p dodai-charts

build-ratatui:
    cargo build -p dodai-ratatui

build-iced:
    cargo build -p dodai-iced

build-gpui:
    cargo build -p dodai-gpui

build-topcoat:
    cargo build -p dodai-topcoat

build-release:
    for c in dodai-core dodai-charts dodai-ratatui dodai-iced dodai-gpui dodai-topcoat; do cargo build --release -p $c || exit 1; done

# ── Run the showcases ─────────────────────────────────────────────────────────

# Ratatui showcase (t = next theme)
run-ratatui:
    cargo run -p dodai-ratatui --example ratatui_showcase

# Iced showcase
run-iced:
    cargo run -p dodai-iced --example iced_showcase

# Bevy showcase (the same pages as the gpui one)
run-bevy:
    cargo run -p dodai-bevy --example bevy_showcase

# Dioxus showcase, served at http://127.0.0.1:3030
run-dioxus:
    cargo run -p dodai-dioxus --example dioxus_showcase

# Topcoat showcase (server-rendered HTML), served at http://127.0.0.1:3031
run-topcoat-live:
    cargo run -p dodai-topcoat --example topcoat_showcase

# GPUI showcase
run-gpui:
    cargo run -p dodai-showcase

# Regenerate every app icon asset from assets/icons/dodai.svg
icons:
    nu scripts/icons.nu --id dodai --gui-assets crates/dodai-showcase/assets

# Write the web showcase to showcase.html and open it (macOS `open`, Linux `xdg-open`)
run-topcoat:
    cargo run -q -p dodai-topcoat --example html > showcase.html
    @(command -v open >/dev/null && open showcase.html) || (command -v xdg-open >/dev/null && xdg-open showcase.html) || echo "Wrote showcase.html"

# ── Test ──────────────────────────────────────────────────────────────────────

test:
    cargo test --workspace --locked --all-targets
    cargo test --workspace --locked --doc

test-core:
    cargo test -p dodai-core --all-features

test-charts:
    cargo test -p dodai-charts

# The dashboard engine, with its JSON Schema feature.
test-dash:
    cargo test -p dodai-dash --all-features

# gpui feature slices must each compile on their own
check-gpui-features:
    cargo check -p dodai-charts --features gpui,material
    cargo check -p dodai-gpui --no-default-features --features components
    cargo check -p dodai-gpui --no-default-features --features frost
    cargo check -p dodai-gpui --no-default-features
    cargo check -p dodai-gpui --no-default-features --features cyberpunk
    cargo check -p dodai-gpui --no-default-features --features material
    cargo check -p dodai-gpui --no-default-features --features components,cyberpunk
    cargo check -p dodai-gpui --no-default-features --features components,material
    cargo check -p dodai-icons --no-default-features
    cargo check -p dodai-icons --no-default-features --features theme
    cargo check -p dodai-icons --no-default-features --features cyberpunk
    cargo check -p dodai-icons --no-default-features --features material
    cargo check -p dodai-icons --no-default-features --features cyberpunk,neobrutalism
    cargo check -p dodai-icons --no-default-features --features symbols-vehicle
    cargo check -p dodai-icons --no-default-features --features symbols-medical
    cargo check -p dodai-icons --no-default-features --features symbols-military
    cargo check -p dodai-icons --no-default-features --features symbols-industrial
    cargo check -p dodai-icons --no-default-features --features symbols-aviation
    cargo check -p dodai-icons --no-default-features --features symbols
    cargo check -p dodai-gpui --no-default-features --features symbols-medical
    cargo check -p dodai-core --features cyberpunk
    cargo check -p dodai-core --features material
    cargo check -p dodai-core --features neobrutalism
    cargo check -p dodai-core --features dodai

test-ratatui:
    cargo test -p dodai-ratatui

test-iced:
    cargo test -p dodai-iced

test-gpui:
    cargo test -p dodai-gpui

test-topcoat:
    cargo test -p dodai-topcoat

# Run Nu script tests
test-nu: _check-nu
    nu scripts/tests/run_all.nu

test-all-nu: test test-nu
    @echo "✅ All Rust and Nu tests passed!"

# ── Code quality ──────────────────────────────────────────────────────────────

check:
    cargo check --workspace --all-targets

fmt:
    cargo fmt --all
    cd crates/dodai-freya && cargo fmt
    cd crates/dodai-xilem && cargo fmt

fmt-check:
    cargo fmt --all -- --check
    cd crates/dodai-freya && cargo fmt --check
    cd crates/dodai-xilem && cargo fmt --check

clippy:
    cargo clippy --workspace --all-targets -- -D warnings

# Everything that must pass before a release (auto-formats first).
check-all: fmt clippy test test-nu
    cargo fmt --all -- --check
    @echo "✅ All checks passed!"

# freya is its own workspace (own Skia, own lock): not covered by the workspace recipes
check-freya:
    cd crates/dodai-freya && cargo fmt --check
    cd crates/dodai-freya && cargo clippy --locked --all-targets -- -D warnings
    cd crates/dodai-freya && cargo test --locked

# xilem is its own workspace (naga 26 clashes with gpui and iced when unified)
check-xilem:
    cd crates/dodai-xilem && cargo fmt --check
    cd crates/dodai-xilem && cargo clippy --locked --all-targets -- -D warnings
    cd crates/dodai-xilem && cargo test --locked

# The fuzz crate is outside the workspace; it must still compile after public struct changes
check-fuzz:
    cd fuzz && cargo check --bins

# Every toolkit, every test: workspace (incl. bevy) + Nu scripts + freya + fuzz. Stops on the first failure.
gate: check-all machete check-freya check-fuzz
    @echo "✅ Full gate passed (all toolkits)!"

# Full pre-release gate: check-all + release build
check-release: check-all build-release
    @echo "✅ Release quality gate passed!"

# ── Docs ──────────────────────────────────────────────────────────────────────

doc:
    cargo doc --workspace --no-deps --open

# ── Changelog ─────────────────────────────────────────────────────────────────

changelog: _check-git-cliff
    git-cliff --output CHANGELOG.md

# Prepend only unreleased commits to CHANGELOG.md
changelog-unreleased: _check-git-cliff
    git-cliff --unreleased --prepend CHANGELOG.md

changelog-preview: _check-git-cliff
    git-cliff --unreleased

changelog-latest: _check-git-cliff
    git-cliff --latest --strip header

release-preview: _check-git-cliff
    git-cliff --unreleased --tag v$(nu scripts/version.nu)

# ── Release ───────────────────────────────────────────────────────────────────

validate-tag tag: _check-nu
    nu scripts/ci/validate_tag.nu {{tag}}

# Bump version everywhere, update changelog, commit and tag (runs the full gate first)
bump version: (validate-tag "v" + version) check-release _check-git-cliff
    nu scripts/bump_version.nu {{version}}

check-publish: _check-nu
    nu scripts/check_publish.nu

# The first release: every crate in dependency order (scripts/lib/versioning.nu); charts is held back
publish: check-all _check-nu
    nu scripts/publish_all.nu

# Package every first-release crate without uploading
publish-dry: _check-nu
    nu scripts/publish_all.nu --dry-run

# Publish charts
publish-charts: check-all
    cargo publish -p dodai-charts

# Dry-run charts
publish-charts-dry: check-all
    cargo publish --dry-run -p dodai-charts

# Line coverage (needs cargo-llvm-cov); writes target/llvm-cov/html
coverage:
    cargo llvm-cov --workspace --exclude dodai-bevy --html
    @echo "→ target/llvm-cov/html/index.html"

# Run the dependency update workflow on GitHub on demand (needs gh)
deps-update-now:
    gh workflow run deps-update.yml

# Bump, then push commit + tag to GitHub (CI publishes and creates the GitHub release)
release version: (bump version)
    git push --follow-tags origin "$(git branch --show-current)"
    @echo "✅ Release v{{version}} pushed to GitHub."

# Bump, commit, tag, then push to every remote.
release-all version: (bump version)
    #!/usr/bin/env sh
    {{PUSHFN}}
    failed=""; skipped=""
    for r in {{REMOTES}}; do
        git remote get-url "$r" >/dev/null 2>&1 || { echo "↷ skipping $r (not configured)"; skipped="$skipped $r"; continue; }
        gpush --follow-tags "$r" "$(git branch --show-current)" || failed="$failed $r"
    done
    [ -n "$skipped" ] && echo "ℹ️  Skipped:$skipped"
    [ -n "$failed" ] && { echo "⚠️  Failed:$failed"; exit 1; }
    echo "✅ Release v{{version}} pushed to: {{REMOTES}}"

# Bump, commit, tag, then push to Gitea Starscream only.
release-gitea-starscream version: (bump version)
    git push --follow-tags gitea-starscream "$(git branch --show-current)"

# Push the latest commit and tags to GitHub (no bump; full gate first).
push-release: check-all
    git push --follow-tags origin "$(git branch --show-current)"

# Push the latest commit and tags to every remote (no bump; continues on failure).
push-release-all: check-all push-all push-tags-all

# Manually re-trigger the GitHub Release workflow for an existing tag.
release-retrigger version:
    @command -v gh >/dev/null 2>&1 || { echo "❌ GitHub CLI (gh) not found. Install from https://cli.github.com"; exit 1; }
    gh workflow run release.yml --field tag=v{{version}}

# ── Git remotes & pushing ─────────────────────────────────────────────────────
# Every *-all recipe loops over REMOTES and keeps going when one fails.

REMOTES := "origin codeberg gitea-starscream gitea-microlab"

# Fail fast (5 s) instead of hanging when a remote host is unreachable.
export GIT_SSH_COMMAND := "ssh -o ConnectTimeout=5 -o BatchMode=yes"

# Unreachable remotes warn and are skipped; real rejections still fail the recipe.
PUSHFN := 'gpush() { out=$(git push "$@" 2>&1); rc=$?; if [ $rc -ne 0 ] && printf "%s" "$out" | grep -Eqi "timed out|could not resolve|connection refused|no route to host|network is unreachable"; then echo "⚠️  $r unreachable - NOT pushed (skipped)"; return 0; fi; [ -n "$out" ] && printf "%s\n" "$out"; return $rc; }'

# Show all configured remotes
remotes:
    @git remote -v

# Add any missing remotes
setup-remotes:
    #!/usr/bin/env sh
    git remote get-url origin >/dev/null 2>&1 || git remote add origin git@github.com:Dodai-Tech/dodai-ui.git
    git remote get-url codeberg >/dev/null 2>&1 || git remote add codeberg ssh://git@codeberg.org/sorinirimies/dodai-ui.git
    git remote get-url gitea-starscream >/dev/null 2>&1 || git remote add gitea-starscream gitea@192.168.1.44:Dodai-Tech/dodai-ui.git
    git remote get-url gitea-microlab >/dev/null 2>&1 || git remote add gitea-microlab ssh://git@192.168.1.204:30009/sorin/dodai-ui.git
    git remote -v

# Push the current branch to GitHub (origin)
push:
    git push origin "$(git branch --show-current)"

# Push the current branch to Codeberg
push-codeberg:
    git push codeberg "$(git branch --show-current)"

# Push the current branch to Gitea Starscream
push-gitea-starscream:
    git push gitea-starscream "$(git branch --show-current)"

# Push the current branch to Gitea Microlab
push-gitea-microlab:
    git push gitea-microlab "$(git branch --show-current)"

# Push the current branch to every remote (continues on failure)
push-all:
    #!/usr/bin/env sh
    {{PUSHFN}}
    failed=""; skipped=""
    for r in {{REMOTES}}; do
        git remote get-url "$r" >/dev/null 2>&1 || { echo "↷ skipping $r (not configured; see: just setup-remotes)"; skipped="$skipped $r"; continue; }
        gpush "$r" "$(git branch --show-current)" || failed="$failed $r"
    done
    [ -n "$skipped" ] && echo "ℹ️  Skipped:$skipped"
    [ -n "$failed" ] && { echo "⚠️  Failed to push to:$failed"; exit 1; }
    echo "✅ Pushed to: {{REMOTES}}"

# Force-push the current branch to every remote (no lease — be sure!)
push-all-force:
    #!/usr/bin/env sh
    {{PUSHFN}}
    failed=""
    for r in {{REMOTES}}; do
        git remote get-url "$r" >/dev/null 2>&1 || continue
        gpush --force "$r" "$(git branch --show-current)" || failed="$failed $r"
    done
    [ -n "$failed" ] && { echo "⚠️  Failed:$failed"; exit 1; }
    echo "✅ Force-pushed"

# Pull the current branch from GitHub (origin)
pull:
    git pull origin "$(git branch --show-current)"

# Pull the current branch from Codeberg
pull-codeberg:
    git pull codeberg "$(git branch --show-current)"

# Pull the current branch from Gitea Starscream
pull-gitea-starscream:
    git pull gitea-starscream "$(git branch --show-current)"

# Push all tags to GitHub
push-tags:
    git push origin --tags

# Push all tags to every remote (continues on failure)
push-tags-all:
    #!/usr/bin/env sh
    {{PUSHFN}}
    failed=""
    for r in {{REMOTES}}; do
        git remote get-url "$r" >/dev/null 2>&1 || continue
        gpush "$r" --tags || failed="$failed $r"
    done
    [ -n "$failed" ] && { echo "⚠️  Failed:$failed"; exit 1; }
    echo "✅ Tags pushed to: {{REMOTES}}"

# Force-sync Gitea Starscream with the local branch + tags
sync-gitea-starscream:
    git push gitea-starscream "$(git branch --show-current)" --force
    git push gitea-starscream --tags --force

# ── Force / per-remote variants (no lease: be sure!) ──────────────────────────

# Force-push the current branch to GitHub (origin)
push-force:
    git push --force origin "$(git branch --show-current)"

# Force-push the current branch to Codeberg
push-codeberg-force:
    git push --force codeberg "$(git branch --show-current)"

# Force-push the current branch to Gitea Starscream
push-gitea-starscream-force:
    git push --force gitea-starscream "$(git branch --show-current)"

# Force-push the current branch to Gitea Microlab
push-gitea-microlab-force:
    git push --force gitea-microlab "$(git branch --show-current)"

# Pull the current branch from Gitea Microlab
pull-gitea-microlab:
    git pull gitea-microlab "$(git branch --show-current)"

# Pull the current branch from every remote (continues on failure)
pull-all:
    #!/usr/bin/env sh
    failed=""
    for r in {{REMOTES}}; do
        git remote get-url "$r" >/dev/null 2>&1 || { echo "↷ skipping $r (not configured)"; continue; }
        git pull "$r" "$(git branch --show-current)" || failed="$failed $r"
    done
    if [ -n "$failed" ]; then echo "⚠️  Failed to pull from:$failed"; exit 1; fi
    echo "✅ Pulled from every configured remote"

# Force-push all tags to GitHub (moves existing tags)
push-tags-force:
    git push --force origin --tags

# Force-push all tags to every remote (continues on failure; moves existing tags)
push-tags-all-force:
    #!/usr/bin/env sh
    {{PUSHFN}}
    failed=""
    for r in {{REMOTES}}; do
        git remote get-url "$r" >/dev/null 2>&1 || { echo "↷ skipping $r (not configured)"; continue; }
        gpush --force "$r" --tags || failed="$failed $r"
    done
    if [ -n "$failed" ]; then echo "⚠️  Failed to force-push tags to:$failed"; exit 1; fi
    echo "✅ Tags force-pushed to every configured remote"

# Force-push the latest commit and tags to every remote, no bump
push-release-all-force: check-all
    #!/usr/bin/env sh
    {{PUSHFN}}
    failed=""
    for r in {{REMOTES}}; do
        git remote get-url "$r" >/dev/null 2>&1 || { echo "↷ skipping $r (not configured)"; continue; }
        gpush --force --follow-tags "$r" "$(git branch --show-current)" || failed="$failed $r"
    done
    if [ -n "$failed" ]; then echo "⚠️  Failed to force-push to:$failed"; exit 1; fi
    echo "✅ Latest commit and tags force-pushed to every configured remote"

# Short name for push-release-all
alias release-push := push-release-all

# Bump, commit, tag, then push to Gitea Microlab only
release-gitea-microlab version: (bump version)
    git push --follow-tags gitea-microlab "$(git branch --show-current)"

# Force-sync Gitea Microlab with the local branch + tags
sync-gitea-microlab:
    git push gitea-microlab "$(git branch --show-current)" --force
    git push gitea-microlab --tags --force

# Update deps, run the gate, then commit + push Cargo.lock if green
update-deps:
    cargo update
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace --locked --all-targets
    git add Cargo.lock
    git diff --cached --quiet || git commit -m "chore: update dependencies"
    git push origin "$(git branch --show-current)"

# ── Misc ──────────────────────────────────────────────────────────────────────

clean:
    cargo clean

update:
    cargo update

outdated:
    cargo outdated --workspace

version: _check-nu
    @nu scripts/version.nu

deny:
    cargo deny check

audit:
    cargo audit

# Unused dependencies in every manifest (workspace, freya, tools, fuzz)
machete:
    cargo machete

# Fuzz a target (needs nightly + cargo-fuzz): just fuzz svg_path
fuzz target="svg_path" secs="60":
    cargo +nightly fuzz run {{target}} -- -max_total_time={{secs}}

# Short fuzz run of every target (needs nightly + cargo-fuzz)
fuzz-smoke secs="10":
    #!/usr/bin/env bash
    set -euo pipefail
    for t in $(cargo +nightly fuzz list); do
        cargo +nightly fuzz run "$t" -- -max_total_time={{secs}}
    done

# Regenerate dist/: W3C design tokens (JSON), CSS custom properties and the SVG icon pack.
export:
    cargo run -q -p dodai-topcoat --example export -- dist

# ── More remote recipes: tags per remote, force pulls, Gitea setup, first release ─────────

# Push all tags to Codeberg
push-tags-codeberg:
    git push codeberg --tags

# Push all tags to Gitea Starscream
push-tags-gitea-starscream:
    git push gitea-starscream --tags

# Push all tags to Gitea Microlab
push-tags-gitea-microlab:
    git push gitea-microlab --tags

# Force-push all tags to Codeberg (moves existing tags)
push-tags-codeberg-force:
    git push --force codeberg --tags

# Force-push all tags to Gitea Starscream (moves existing tags)
push-tags-gitea-starscream-force:
    git push --force gitea-starscream --tags

# Force-push all tags to Gitea Microlab (moves existing tags)
push-tags-gitea-microlab-force:
    git push --force gitea-microlab --tags

# Make the local branch equal GitHub (origin): fetch, then hard reset. DISCARDS local commits!
pull-force:
    git fetch origin
    git reset --hard "origin/$(git branch --show-current)"

# Make the local branch equal Gitea Starscream. DISCARDS local commits!
pull-gitea-starscream-force:
    git fetch gitea-starscream
    git reset --hard "gitea-starscream/$(git branch --show-current)"

# Make the local branch equal Gitea Microlab. DISCARDS local commits!
pull-gitea-microlab-force:
    git fetch gitea-microlab
    git reset --hard "gitea-microlab/$(git branch --show-current)"

# Fetch every remote and make the local branch equal GitHub (origin is the source of truth). DISCARDS local commits!
pull-all-force:
    #!/usr/bin/env sh
    for r in {{REMOTES}}; do
        git remote get-url "$r" >/dev/null 2>&1 || { echo "↷ skipping $r (not configured)"; continue; }
        git fetch "$r" || echo "⚠️  could not fetch $r"
    done
    git reset --hard "origin/$(git branch --show-current)"

# Force-sync every Gitea instance with the local branch + tags (continues on failure)
sync-all-gitea:
    #!/usr/bin/env sh
    {{PUSHFN}}
    failed=""
    for r in gitea-starscream gitea-microlab; do
        git remote get-url "$r" >/dev/null 2>&1 || { echo "↷ skipping $r (not configured)"; continue; }
        gpush --force "$r" "$(git branch --show-current)" || failed="$failed $r"
        gpush --force "$r" --tags || failed="$failed $r-tags"
    done
    [ -n "$failed" ] && { echo "⚠️  Failed to sync:$failed"; exit 1; }
    echo "✅ Every Gitea instance force-synced"

# Add a Gitea remote and optionally push — interactive (nu script)
setup-gitea url: _check-nu
    nu scripts/setup_gitea.nu {{ url }}

# Migrate this project to dual GitHub + Gitea hosting (interactive)
migrate-gitea: _check-nu
    nu scripts/migrate_to_gitea.nu

# First release of the version already in Cargo.toml: gate, changelog, tag, push to GitHub only
release-initial: check-release _check-git-cliff _check-nu (_release-initial "origin")

# Same as release-initial, but pushes to every remote
release-initial-all: check-release _check-git-cliff _check-nu (_release-initial REMOTES)

[private]
_release-initial remotes:
    #!/usr/bin/env sh
    remotes="{{ remotes }}"
    set -e
    version=$(nu scripts/version.nu)
    tag="v$version"
    nu scripts/ci/validate_tag.nu "$tag" >/dev/null
    if git rev-parse -q --verify "refs/tags/$tag" >/dev/null; then
        echo "❌ Tag $tag already exists. Use 'just release <new-version>' to bump."; exit 1
    fi
    if [ -n "$(git status --porcelain)" ]; then
        echo "❌ Working tree is dirty — commit or stash first."; exit 1
    fi
    git-cliff --output CHANGELOG.md --tag "$tag"
    git add CHANGELOG.md
    git diff --cached --quiet || git commit -m "chore: release $tag"
    git tag -a "$tag" -m "Release $tag"
    echo "🏷️  Tagged $tag — pushing to: $remotes"
    failed=""; skipped=""
    for r in $remotes; do
        git remote get-url "$r" >/dev/null 2>&1 || { echo "↷ skipping $r (not configured; see: just setup-remotes)"; skipped="$skipped $r"; continue; }
        git push "$r" "$(git branch --show-current)" || failed="$failed $r"
        git push "$r" "$tag" || failed="$failed $r"
    done
    [ -n "$skipped" ] && echo "ℹ️  Skipped:$skipped"
    [ -n "$failed" ] && { echo "⚠️  Release $tag failed to push to:$failed"; exit 1; }
    echo "✅ Release $tag pushed to: $remotes — Release workflows will start."

# Regenerate the design-tool files: tokens, component catalog, and the OpenPencil/Figma .fig.
design:
    cargo run -q -p dodai-topcoat --example export -- dist
    rm -rf crates/dist
    cargo run -q -p dodai-showcase-model --example plots -- dist/plots
    cargo run -q -p dodai-icons --example design_icons -- dist/icons-design
    cargo run -q -p dodai-showcase-model --example surfaces -- dist/surfaces
    nu scripts/design.nu catalog
    nu scripts/design.nu build
    nu scripts/design.nu spec
    nu scripts/design.nu verify
