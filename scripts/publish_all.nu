#!/usr/bin/env nu
# Publish every first-release crate in dependency order (the order lives in
# scripts/lib/versioning.nu). A crate whose version is already on crates.io is skipped, so a
# failed run can simply be started again. Needs `cargo login` or CARGO_REGISTRY_TOKEN.
# Usage: nu scripts/publish_all.nu            publish
#        nu scripts/publish_all.nu --dry-run  package everything (nothing is uploaded)

use lib/versioning.nu *

def main [--dry-run] {
    let order = (publish-order)
    if $dry_run {
        # Dependents can only be verified once their dependencies are live, so package without
        # verifying; the CI slices already build every crate.
        run-external "cargo" "package" "--no-verify" "--allow-dirty" ...($order | each {|c| ["-p" $c] } | flatten)
        return
    }
    for c in $order {
        nu scripts/ci/publish_crate.nu $c
        sleep 30sec   # let the crates.io index catch up before dependants publish
    }
}
