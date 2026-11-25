#!/usr/bin/env nu
# Compares renderings of the same components from two sources, by numbers (no one reads images).
#
#   nu scripts/compare.nu <dir-a> <dir-b> [--max-blocks 5] [--out dist/compare/diff]
#
# Both directories hold `<name>.png` files with matching names (for example a gpui capture and the
# design file's export of the same frame). Prints one line per pair: pixels and layout cells that
# differ, plus the size when it does not match, and exits 1 when a pair passes `--max-blocks`
# percent of cells, has another size, or exists on one side only. Differences are painted into
# `--out` (red: colour, blue: size) for whoever wants to look.
def main [
    a: path
    b: path
    --max-blocks: float = 100.0
    --out: path = "dist/compare/diff"
] {
    cargo run -q -p dodai-diff -- $a $b --max-blocks ($max_blocks | into string) --out $out
}
