#!/usr/bin/env nu
# Build the design-tool files (OpenPencil / Figma `.fig`) from the same sources as the code.
#
#   nu scripts/design.nu catalog   write dist/design/components.json (every component and the toolkits that have it)
#   nu scripts/design.nu build     write dist/design/dodai.fig (tokens as variables, component catalog, Button)
#   nu scripts/design.nu check     fail when dist/design/components.json is stale (CI)
#
# Tokens come from `cargo run -p dodai-ui-topcoat --example export -- dist` (dist/design/<system>/<theme>.tokens.json):
# a folder is a variable collection, a file a mode. Needs bun for `build` (runs `@open-pencil/cli`).

use lib/parity.nu [component-matrix, toolkits]

def catalog-json []: nothing -> string {
    {
        toolkits: (toolkits | get id)
        components: (component-matrix)
    } | to json --indent 2
}

def "main catalog" [] {
    catalog-json | save --force dist/design/components.json
    print "wrote dist/design/components.json"
}

def "main check" [] {
    let want = (catalog-json)
    if (not ("dist/design/components.json" | path exists)) or ((open --raw dist/design/components.json) != $want) {
        error make { msg: "dist/design/components.json is stale: run `nu scripts/design.nu catalog` and commit it" }
    }
    print "design catalog: up to date"
}

def "main build" [--out: string = "dist/design/dodai.fig"] {
    let tmp = (mktemp --directory)
    let base = $"($tmp)/base.html"
    "<div></div>" | save --force $base
    let fig = $"($tmp)/dodai.fig"
    ^bunx @open-pencil/cli import $base -o $fig | ignore

    # One import call per system: each folder is a collection, each file a mode.
    for entry in (ls dist/design | where type == dir | sort-by name) {
        let dir = ($entry.name | path expand)
        let system = ($dir | path basename)
        let files = (glob $"($dir)/*.tokens.json" | sort | each { |f|
            { path: $"($system)/($f | path basename)", text: (open --raw $f) }
        })
        { files: $files } | to json | save --force $"($tmp)/args.json"
        let res = (^bunx @open-pencil/cli tool call import_design_tokens $fig --args-file $"($tmp)/args.json" --write | lines | where { $in !~ '^Written to' } | str join "\n" | from json)
        print $"($system): ($res.added) variables, ($res.skipped | length) skipped"
    }

    let js = $"const CATALOG = (open --raw dist/design/components.json);\n(open --raw scripts/design/build.js)"
    print ($js | ^bunx @open-pencil/cli eval $fig --stdin --write)
    cp $fig $out
    print $"wrote ($out)"
}

def main [] {
    print "usage: nu scripts/design.nu catalog | build | check"
}
