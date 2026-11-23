#!/usr/bin/env nu
# Build the design-tool files (OpenPencil / Figma `.fig`) from the same sources as the code.
#
#   nu scripts/design.nu catalog   write dist/design/components.json (every component and the toolkits that have it)
#   nu scripts/design.nu build     write dist/design/dodai.fig (tokens as variables, component catalog, Button)
#   nu scripts/design.nu spec      write dist/design/spec/<system>.md, the written spec of every design system
#   nu scripts/design.nu verify    fail when the built .fig does not carry exactly the token variables
#   nu scripts/design.nu check     fail when dist/design/components.json is stale (CI)
#
# Tokens come from `cargo run -p dodai-topcoat --example export -- dist` (dist/design/<system>/<theme>.tokens.json):
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
    for f in (spec-files) {
        if (not ($f.path | path exists)) or ((open --raw $f.path) != $f.text) {
            error make { msg: $"($f.path) is stale: run `nu scripts/design.nu spec` and commit it" }
        }
    }
    print "design catalog and spec: up to date"
}

def "main build" [--out: string = "dist/design/dodai.fig"] {
    let tmp = (mktemp --directory)
    let base = $"($tmp)/base.html"
    "<div></div>" | save --force $base
    let fig = $"($tmp)/dodai.fig"
    ^bunx @open-pencil/cli import $base -o $fig | ignore

    # One import call per system: each folder is a collection, each file a mode.
    for entry in (ls dist/design | where type == dir | where { ($in.name | path basename) != "spec" } | sort-by name) {
        let dir = ($entry.name | path expand)
        let system = ($dir | path basename)
        let files = (glob $"($dir)/*.tokens.json" | sort | each { |f|
            { path: $"($system)/($f | path basename)", text: (open --raw $f) }
        })
        { files: $files } | to json | save --force $"($tmp)/args.json"
        let res = (^bunx @open-pencil/cli tool call import_design_tokens $fig --args-file $"($tmp)/args.json" --write | lines | where { $in !~ '^Written to' } | str join "\n" | from json)
        print $"($system): ($res.added) variables, ($res.skipped | length) skipped"
    }

    let plots = (glob dist/plots/*.json | sort | each { |f| $"\"($f | path parse | get stem)\": (open --raw $f)" } | str join ",")
    let js = $"const CATALOG = (open --raw dist/design/components.json);\nconst PLOTS = {($plots)};\n(open --raw scripts/design/build.js)"
    print ($js | ^bunx @open-pencil/cli eval $fig --stdin --write)
    cp $fig $out
    print $"wrote ($out)"
}

# Flatten one token group to rows { name, value }.
def rows [group: record, prefix: string = ""]: nothing -> list<record> {
    $group | transpose name tok | each { |r|
        let path = (if $prefix == "" { $r.name } else { $"($prefix)/($r.name)" })
        if ("$value" in ($r.tok | columns)) {
            let v = $r.tok."$value"
            [{ name: $path, value: (if ($v | describe | str starts-with "record") { $v | to json --raw } else { $v | into string }) }]
        } else {
            rows $r.tok $path
        }
    } | flatten
}

def table-md [group: record]: nothing -> string {
    let r = (rows $group)
    if ($r | is-empty) { return "" }
    ["| token | value |", "| --- | --- |"] | append ($r | each { |x| $"| `($x.name)` | `($x.value)` |" }) | str join "\n"
}

# component name -> gpui source file (the reference implementation), one scan for all.
def gpui-sources []: nothing -> record {
    ^rg -o --no-heading --no-line-number '^pub (struct|enum|fn|type) [A-Z][A-Za-z0-9]*' crates/dodai-gpui/src | lines
        | each { |l| let p = ($l | parse -r '^(?<file>[^:]+):pub \w+ (?<name>\w+)$'); if ($p | is-empty) { null } else { $p | first } }
        | compact | reduce --fold {} { |r, acc| if ($acc | get -o $r.name | is-empty) { $acc | insert $r.name $r.file } else { $acc } }
}

# Every chart of the gpui chart set (a `#[derive(IntoElement)]` struct), with its summary and source.
def gpui-charts []: nothing -> list<record> {
    glob crates/dodai-charts/src/gpui/*.rs | sort | each { |f|
        let rel = ($f | path relative-to (pwd))
        open --raw $f | parse --regex '(?<doc>(?:/// [^\n]*\n)+)#\[derive\([^)]*IntoElement[^)]*\)\]\npub struct (?<name>\w+)'
            | each { |m| { name: $m.name, file: $rel, doc: ($m.doc | lines | first | str replace '/// ' '') } }
    } | flatten
}

def spec-md [system: string, base: record, themes: list<string>]: nothing -> string {
    let comps = (open dist/design/components.json)
    let src = (gpui-sources)
    let charts = (gpui-charts)
    let groups = [
        [color "Colour roles (default theme)"] [font Typography] [space Spacing] [shape Shape] [effect Effects]
        [motion Motion] [surface "Surface treatments"] [size "Control sizes"] [look "Look switches"] [nav Navigation]
    ]
    let sections = ($groups | each { |g|
        if ($base | get -o $g.0 | is-empty) { "" } else { $"## ($g.1)\n\n(table-md ($base | get $g.0))\n" }
    })
    let kits = ($base.kit | columns | each { |c| $c | split row '-' | first } | uniq)
    let btn = ($base.button | columns | each { |c| $c | str replace -r '-(bg|fg|border)$' '' } | uniq)
    [
        $"# (($system | str capitalize)) design system"
        ""
        $"Generated by `nu scripts/design.nu spec` from `dist/design/($system)/` and the gpui reference implementation. Do not edit by hand."
        ""
        $"Themes \(modes): ($themes | each { |t| $'`($t)`' } | str join ', ')."
        ""
        ...$sections
        $"## Button \(($btn | length) variant/state looks\)\n\n($btn | each { |b| $'`($b)`' } | str join ', ')\n"
        $"## Surface families \(Kit, ($kits | length)\)\n\n($kits | each { |b| $'`($b)`' } | str join ', ')\n"
        $"## Charts \(($charts | length) in the gpui chart set\)\n"
        "Series colours are the `color/series/*` roles; plot sizes come from the `size/chart-*` tokens. The design file draws every chart of the showcase on the `<system> / Charts` page from the showcase model's plots (`just design`)."
        ""
        "| chart | summary | gpui reference |"
        "| --- | --- | --- |"
        ...($charts | each { |c| $"| ($c.name) | ($c.doc) | `($c.file)` |" })
        ""
        $"## Components \(($comps.components | length)\)\n"
        "| component | toolkits | gpui reference |"
        "| --- | --- | --- |"
        ...($comps.components | each { |c| let src = ($src | get -o $c.name | default "-"); $"| ($c.name) | ($c.toolkits | str join ', ') | `($src)` |" })
    ] | str join "\n"
}

def spec-files []: nothing -> list<record> {
    ls dist/design | where type == dir | where { ($in.name | path basename) != "spec" } | sort-by name | each { |entry|
        let system = ($entry.name | path basename)
        let files = (glob $"($entry.name | path expand)/*.tokens.json" | sort)
        let themes = ($files | each { |f| $f | path basename | str replace '.tokens.json' '' })
        let base = (open $"dist/design/($system)/($system).tokens.json")
        { path: $"dist/design/spec/($system).md", text: (spec-md $system $base $themes), system: $system, base: $base }
    }
}

def "main spec" [] {
    mkdir dist/design/spec
    for f in (spec-files) { $f.text | save --force $f.path; print $"wrote ($f.path)" }
}

# Count the token leaves of a group (a node with `$value` is a leaf).
def leaves [node: record]: nothing -> int {
    if ("$value" in ($node | columns)) { 1 } else { $node | values | each { |v| if ($v | describe | str starts-with "record") { leaves $v } else { 0 } } | math sum }
}

# The .fig must carry exactly the variables the token files define: the link between the
# OpenPencil design and the implementation (the tokens are what every toolkit reads).
def "main verify" [--fig: string = "dist/design/dodai.fig"] {
    let want = (spec-files | each { |f| leaves $f.base } | math sum)
    let got = (^openpencil variables $fig | lines | where { $in =~ '^\[[0-9]+\]' } | length)
    if $want != $got {
        error make { msg: $"($fig) has ($got) variables, the token files define ($want): run `just design`" }
    }
    print $"design file: ($got) variables match the token files"
}

def main [] {
    print "usage: nu scripts/design.nu catalog | build | spec | verify | check"
}
