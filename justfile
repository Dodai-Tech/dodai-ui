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
    @echo "✅ All tools installed!"

# ── Build ─────────────────────────────────────────────────────────────────────

# Build the whole workspace (dev). Per-crate: `cargo build --workspace` cannot unify
# iced's naga 27 with gpui's naga 25 (see dodai-iced dev-dependencies).
build:
    for c in dodai-core dodai-ratatui dodai-iced dodai-gpui dodai-topcoat; do cargo build -p $c || exit 1; done

build-core:
    cargo build -p dodai-core

build-ratatui:
    cargo build -p dodai-ratatui

build-iced:
    cargo build -p dodai-iced

build-gpui:
    cargo build -p dodai-gpui

build-topcoat:
    cargo build -p dodai-topcoat

build-release:
    for c in dodai-core dodai-ratatui dodai-iced dodai-gpui dodai-topcoat; do cargo build --release -p $c || exit 1; done

# ── Run the showcases ─────────────────────────────────────────────────────────

# Ratatui showcase (t = next theme)
run-ratatui:
    cargo run -p dodai-ratatui --example ratatui_showcase

# Iced showcase
run-iced:
    cargo run -p dodai-iced --example iced_showcase

# GPUI showcase
run-gpui:
    cargo run -p dodai-gpui --example gpui_showcase

# Write the web showcase to showcase.html and open it (macOS `open`, Linux `xdg-open`)
run-topcoat:
    cargo run -q -p dodai-topcoat --example html > showcase.html
    @(command -v open >/dev/null && open showcase.html) || (command -v xdg-open >/dev/null && xdg-open showcase.html) || echo "Wrote showcase.html"

# ── Test ──────────────────────────────────────────────────────────────────────

test:
    cargo test --workspace --locked --all-targets
    cargo test --workspace --locked --doc

test-core:
    cargo test -p dodai-core

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

fmt-check:
    cargo fmt --all -- --check

clippy:
    cargo clippy --workspace --all-targets -- -D warnings

# Everything that must pass before a release (auto-formats first).
check-all: fmt clippy test test-nu
    cargo fmt --all -- --check
    @echo "✅ All checks passed!"

# Full pre-release gate: check-all + release build
check-release: check-all build-release
    @echo "✅ Release quality gate passed!"

# ── Docs ──────────────────────────────────────────────────────────────────────

doc:
    cargo doc --workspace --no-deps --open

# ── Changelog ─────────────────────────────────────────────────────────────────

changelog: _check-git-cliff
    git-cliff --output CHANGELOG.md

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

# Publish all crates in dependency order (core first)
publish: check-all
    cargo publish -p dodai-core
    @echo "⏳ waiting for crates.io index…" && sleep 30
    cargo publish -p dodai-ratatui
    cargo publish -p dodai-iced
    cargo publish -p dodai-gpui
    cargo publish -p dodai-topcoat

# Bump, then push commit + tag (CI publishes and creates the GitHub release)
release version: (bump version)
    git push origin HEAD --follow-tags

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
