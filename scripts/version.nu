#!/usr/bin/env nu
# Prints the current workspace version from Cargo.toml.
# Usage: nu scripts/version.nu

use lib/versioning.nu *
read-version (open Cargo.toml --raw) | print
