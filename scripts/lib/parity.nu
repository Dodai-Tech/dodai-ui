# Pure helpers and source readers for scripts/parity.nu (unit-tested in scripts/tests).

# Toolkits, reference first. `model` = the example renderer that walks the showcase model.
export def toolkits []: nothing -> table {
    [
        { id: gpui,    crate: dodai-gpui,    kind: rust,  model: "" }
        { id: iced,    crate: dodai-iced,    kind: rust,  model: "crates/dodai-iced/examples/iced_showcase/render.rs" }
        { id: dioxus,  crate: dodai-dioxus,  kind: rust,  model: "crates/dodai-dioxus/examples/dioxus_showcase/render.rs" }
        { id: bevy,    crate: dodai-bevy,    kind: rust,  model: "crates/dodai-bevy/examples/bevy_showcase/render.rs" }
        { id: topcoat, crate: dodai-topcoat, kind: css,   model: "crates/dodai-topcoat/examples/topcoat_showcase/render.rs" }
        { id: egui,    crate: dodai-egui,    kind: rust,  model: "crates/dodai-egui/examples/egui_showcase/render.rs" }
        { id: freya,   crate: dodai-freya,   kind: rust,  model: "" }
        { id: slint,   crate: dodai-slint,   kind: slint, model: "" }
        { id: leptos,  crate: dodai-leptos,  kind: rust,  model: "" }
        { id: yew,     crate: dodai-yew,     kind: rust,  model: "" }
        { id: tui,      crate: dodai-tui, kind: rust,  model: "" }
        { id: xilem,   crate: dodai-xilem,   kind: rust,  model: "" }
    ]
}

# Variant names of `pub enum Node` in the model's node.rs text.
export def node-variants [src: string]: nothing -> list<string> {
    let body = ($src | split row "pub enum Node {" | get 1 | split row "\n}\n" | first)
    $body | lines | where { $in =~ '^    [A-Z]' } | each { $in | str trim | parse --regex '^([A-Za-z0-9]+)' | get capture0.0 } | uniq
}

# Variants a renderer's text names (`Demo::X` or `Node::X`).
export def matched-variants [src: string, variants: list<string>]: nothing -> list<string> {
    $variants | where { |v| $src =~ ("(?:Demo|Node)::" + $v + "\\b") }
}

# Public component names of the reference toolkit: pub structs that are not plumbing.
export def component-names [src: string]: nothing -> list<string> {
    $src
    | parse --regex '(?m)^pub struct ([A-Z][A-Za-z0-9]*)'
    | get capture0
    | where { $in !~ '(State|Config|Props|Options|Kind|Mode|Id|Style|Tokens|Ctx|Handle|Group|Colours?)$' }
    | uniq
    | sort
}

export def snake [name: string]: nothing -> string {
    $name | str replace --all --regex '([a-z0-9])([A-Z])' '${1}_${2}' | str lowercase
}

# Does a toolkit's source expose `name`? Rust: a pub item of that name (type or snake_case fn);
# Slint: a `component`; CSS: a `.dodai-<kebab>` class.
export def exposes [src: string, name: string, kind: string]: nothing -> bool {
    let s = (snake $name)
    let k = ($s | str replace --all "_" "-")
    let pub_item = ("pub (?:struct|enum|fn|type|trait) (?:" + $name + "|" + $s + ")\\b")
    match $kind {
        "css" => (($src =~ ("\\.dodai-" + $k + "\\b")) or ($src =~ $pub_item)),
        "slint" => ($src =~ ("(?:component|struct|enum) L?" + $name + "\\b")),
        _ => ($src =~ $pub_item),
    }
}

# Count occurrences of a token accessor in a source text.
export def uses [src: string, pattern: string]: nothing -> int {
    $src | parse --regex $pattern | length
}

# Concatenated text of a crate's sources (`src`, plus the stylesheet for topcoat).
def crate-src [crate: string, kind: string]: nothing -> string {
    let exts = if $kind == "slint" { ["rs" "slint"] } else { ["rs" "css"] }
    $exts
    | each { |e| glob $"crates/($crate)/{src,ui}/**/*.($e)" }
    | flatten
    | sort
    | each { open --raw $in }
    | str join "\n"
}

def pct [n: int, total: int]: nothing -> string {
    $"(($n * 100 / $total) | math round)%"
}

# Node variants a model renderer fails to name; one line per problem.
export def model-gaps []: nothing -> list<string> {
    let variants = (node-variants (open --raw crates/dodai-showcase-model/src/node.rs))
    toolkits | where model != "" | each { |t|
        let got = (matched-variants (open --raw $t.model) $variants)
        $variants | where { $in not-in $got } | each { $"($t.id): ($in)" }
    } | flatten
}

# The whole docs/PARITY.md text.
export def render-parity []: nothing -> string {
    let variants = (node-variants (open --raw crates/dodai-showcase-model/src/node.rs))
    let gpui_src = (crate-src dodai-gpui rust)
    let comps = (component-names $gpui_src)
    let rows = (toolkits | each { |t|
        let src = (crate-src $t.crate $t.kind)
        let have = if $t.id == gpui { $comps } else { $comps | where { |c| exposes $src $c $t.kind } }
        let nodes = if $t.model == "" { null } else { (matched-variants (open --raw $t.model) $variants) | length }
        {
            toolkit: $t.id
            components: $"($have | length)/($comps | length) \((pct ($have | length) ($comps | length))\)"
            showcase: (if $t.id == gpui { "reference (direct widgets)" } else if $nodes == null { "own page" } else { $"model, ($nodes)/($variants | length) nodes" })
            sizes: (uses $src 'sizes\(\)|size_entries|--sz-')
            look: (uses $src 'look\(\)|--look-')
            nav: (uses $src 'navigation\(\)|--nav-')
            motion: (uses $src 'motion::|Easing|transition:|@keyframes|animation:|animate\(|Animation')
            canvas: (uses $src 'root_style|backdrop_image|paint_canvas|backdrop_view|paint_backdrop|UiBackdrop|fn root|backdrop\(')
            keys: (uses $src 'roving::|Key(?:board)?Event|on_key|keydown|ArrowLeft|key-pressed')
            missing: ($comps | where { $in not-in $have })
        }
    })
    let table = ($rows | reject missing | to md --pretty)
    let details = ($rows | where toolkit != gpui | each { |r|
        let list = if ($r.missing | is-empty) { "nothing" } else { $r.missing | each { $"`($in)`" } | str join ", " }
        $"<details><summary>($r.toolkit): ($r.missing | length) missing</summary>\n\n($list)\n\n</details>\n"
    } | str join "\n")
    [
        "# Toolkit parity"
        ""
        "Generated by `nu scripts/parity.nu` from source. Do not edit; CI fails when it is stale."
        ""
        "* **components**: public gpui components (the reference) that the toolkit also exposes by name."
        "* **showcase**: `model` toolkits draw the whole showcase model (a new `Node` variant will not compile"
        "  until they handle it); `own page` ones hand-write a smaller page."
        "* **sizes / look / nav**: how often the toolkit reads the `Sizes`, `Look` and navigation tokens."
        "* **motion**: uses of the shared motion system (`dodai_core::motion`), CSS transitions/keyframes or animation APIs."
        "* **canvas**: uses of the design's gradient canvas (the mesh of Dodai): the app root helper or backdrop call. Zero means a Dodai window shows a flat colour."
        "* **keys**: uses of the shared roving-focus rule (`dodai_core::roving`) or keyboard event handling."
        "  Both are rough counts, not coverage: they show a toolkit is at zero, not that a non-zero one is complete."
        ""
        $table
        ""
        "## Missing components by toolkit"
        ""
        $details
    ] | str join "\n"
}

# Components with no visual of their own: they wrap, time, cull or observe other components.
# The design file lists them in a spec table instead of drawing them.
export def behaviour-names []: nothing -> list<string> {
    ["DodaiTheme" "EntryScope" "Feed" "Follow" "Interact" "Lazy" "Live" "Overscroll" "OverscrollFeel" "PageGrid"]
}

# First doc-comment paragraph of every `pub struct` in a source text: { name: text }.
export def component-docs [src: string]: nothing -> record {
    $src
    | parse --regex '(?m)(?<doc>(?:^[ \t]*///[^\n]*\n)+)(?:[ \t]*#\[[^\n]*\n)*pub struct (?<name>[A-Z][A-Za-z0-9]*)'
    | reduce --fold {} { |r, acc|
        let line = ($r.doc | lines | each { |l| $l | str trim | str replace --regex '^///\s?' '' | str trim } | take while { |l| $l != "" } | str join ' ')
        if ($acc | get -o $r.name | is-empty) and ($line != "") { $acc | insert $r.name $line } else { $acc }
    }
}

# Every reference component with the toolkits that expose it, its summary and whether it draws
# anything: `{ name, doc, behaviour, toolkits: [id] }`.
export def component-matrix []: nothing -> table {
    let gpui = (crate-src dodai-gpui rust)
    let comps = (component-names $gpui)
    let docs = (component-docs $gpui)
    let behaviour = (behaviour-names)
    let have = (toolkits | each { |t|
        let src = (crate-src $t.crate $t.kind)
        {
            id: $t.id
            has: (if $t.id == gpui { $comps } else { $comps | where { |c| exposes $src $c $t.kind } })
        }
    })
    $comps | each { |c|
        {
            name: $c
            doc: ($docs | get -o $c | default "")
            behaviour: ($c in $behaviour)
            toolkits: ($have | where { $c in $in.has } | get id)
        }
    }
}
