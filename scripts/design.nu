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
        # The design's own theme first: it becomes the collection's default mode, the one the
        # baked shapes (charts, icons, textures) are drawn in.
        let all = (glob $"($dir)/*.tokens.json" | sort)
        let own = ($all | where { ($in | path basename) == $"($system).tokens.json" })
        let rest = ($all | where { ($in | path basename) != $"($system).tokens.json" })
        let files = (($own | append $rest) | each { |f|
            { path: $"($system)/($f | path basename)", text: (open --raw $f) }
        })
        { files: $files } | to json | save --force $"($tmp)/args.json"
        let res = (^bunx @open-pencil/cli tool call import_design_tokens $fig --args-file $"($tmp)/args.json" --write | lines | where { $in !~ '^Written to' } | str join "\n" | from json)
        print $"($system): ($res.added) variables, ($res.skipped | length) skipped"
    }

    let plots = (glob dist/plots/*.json | sort | each { |f| $"\"($f | path parse | get stem)\": (open --raw $f)" } | str join ",")
    let icons = (["stroked" "material" "bold" "symbols"] | each { |n|
        let body = (open --raw $"dist/icons-design/($n).json")
        $"\"($n)\": ($body)"
    } | str join ",")
    let surfaces = (glob dist/surfaces/*.json | sort | each { |f|
        let body = (open --raw $f)
        $"\"($f | path parse | get stem)\": ($body)"
    } | str join ",")
    let shaders = (glob dist/shaders/shaders-*.json | sort | each { |f|
        let body = (open --raw $f)
        $"\"($f | path parse | get stem | str replace 'shaders-' '')\": ($body)"
    } | str join ",")
    let js = $"const SHADERS_BY = {($shaders)};\nconst SURFACES = {($surfaces)};\nconst ICONS = {($icons)};\nconst CATALOG = (open --raw dist/design/components.json);\nconst MOTION = (open --raw dist/design/motion.json);\nconst PLOTS = {($plots)};\n(open --raw scripts/design/build.js)"
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
    glob crates/dodai-gpui/src/**/*.rs | sort | each { |f|
        let rel = ($f | path relative-to (pwd))
        open --raw $f | lines | parse --regex '^pub (?:struct|enum|fn|type) (?<name>[A-Z][A-Za-z0-9]*)' | each { |m| { name: $m.name, file: $rel } }
    } | flatten
    | reduce --fold {} { |r, acc| if ($acc | get -o $r.name | is-empty) { $acc | insert $r.name $r.file } else { $acc } }
}

# Every chart of the gpui chart set (a `#[derive(IntoElement)]` struct), with its summary and source.
def gpui-charts []: nothing -> list<record> {
    glob crates/dodai-charts/src/gpui/*.rs | sort | each { |f|
        let rel = ($f | path relative-to (pwd))
        open --raw $f | parse --regex '(?<doc>(?:///[^\n]*\n)+)#\[derive\([^)]*IntoElement[^)]*\)\]\npub struct (?<name>\w+)'
            | each { |m| { name: $m.name, file: $rel, doc: ($m.doc | lines | first | str replace '/// ' '') } }
    } | flatten
}

# The icon pack a design draws (what `dodai-icons` resolves for its `Look::icons`).
def icon-pack [system: string]: nothing -> string {
    match $system {
        "cyberpunk" => "Stroked (hand-drawn, curve-free)"
        "neobrutalism" => "Bold (Phosphor bold)"
        _ => "Material Symbols"
    }
}

def spec-md [system: string, base: record, themes: list<string>]: nothing -> string {
    let comps = (open dist/design/components.json)
    let src = (gpui-sources)
    let charts = (gpui-charts)
    let motion = (open dist/design/motion.json)
    let behaviour = ($comps.components | where behaviour)
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
        "## Design file pages\n"
        $"The `dist/design/dodai.fig` collection `($system)` has one mode per theme. Pages are named `<System> / <Page>`:\n"
        "| page | what it draws |"
        "| --- | --- |"
        "| Foundations | colour roles, type scale, spacing, radius and the design's shape |"
        "| Button | every variant and state |"
        "| Surfaces | one component set per Kit family |"
        "| Controls | Switch, Checkbox, Radio, Slider, Progress, Input, Tab |"
        "| Feedback | Badge, Alert, Chip, Card, Avatar, Snackbar, Dialog |"
        "| Components | the other showcase components and the hover, focus, error and disabled states |"
        "| Charts | every chart of the showcase model |"
        $"| Icons | every icon of the ((icon-pack $system)) pack |"
        "| Symbols | the dashboard symbol groups |"
        "| Surfaces & glass | the design's frosted glass, and the textures and effects it owns |"
        "| Shaders | a thumbnail of every shader of the catalogue |"
        "| Motion | the duration scale, every easing curve, every entry effect and the overscroll feels |"
        "| Behaviour | the components that draw nothing themselves: what each does |"
        ""
        $"## Motion\n\nThe duration scale in ms: ($motion.durations | each { |d| [$d.name ' ' ($d.ms | into string)] | str join '' | $"`($in)`" } | str join ', ').\n"
        "| easing | group | css |"
        "| --- | --- | --- |"
        ...($motion.easings | each { |e| $"| `($e.name)` | ($e.group) | `($e.css | str substring 0..60)` |" })
        ""
        $"Entry effects: ($motion.entries | get name | str join ', '). Overscroll feels: ($motion.overscroll | each { |f| [$f.name ': max ' ($f.max | into string) ' px, stiffness ' ($f.stiffness | into string) ', damping ' ($f.damping | into string)] | str join '' } | str join '; '). The design file draws all of them on the `<System> / Motion` page."
        ""
        $"## Behaviour \(($behaviour | length) components with no visual\)\n"
        "| component | what it does | toolkits |"
        "| --- | --- | --- |"
        ...($behaviour | each { |c| $"| ($c.name) | ($c.doc) | ($c.toolkits | str join ', ') |" })
        ""
        $"## Components \(($comps.components | length)\)\n"
        "| component | summary | toolkits | gpui reference |"
        "| --- | --- | --- | --- |"
        ...($comps.components | each { |c| let src = ($src | get -o $c.name | default "-"); $"| ($c.name) | ($c.doc | str replace --all '|' '/') | ($c.toolkits | str join ', ') | `($src)` |" })
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
