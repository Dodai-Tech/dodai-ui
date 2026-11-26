#!/usr/bin/env nu
# Compares the gpui components with the design file's frames, by numbers.
#
#   nu scripts/compare-frames.nu [--design material] [--max-blocks 100]
#
# Runs `examples/frames` (a grid of components in one window, theme = the design), screenshots its
# window, exports the design file's component frames of the same names, crops the screenshot to the
# size of each frame and prints how far apart they are (see `dodai-diff`). macOS only: needs a
# screen and Screen Recording permission. Output and differences go to `dist/compare/<design>`.
def main [
    --design: string = "material"
    --max-blocks: float = 100.0
    --wait: duration = 5sec
    --probe # self-test: magenta squares at the cell origins, checks the crop alignment only
] {
    let out = $"dist/compare/($design)"
    rm -rf $out
    mkdir $"($out)/design" $"($out)/diff"
    cargo build -q -p dodai-showcase --example frames
    let manifest = $"($out)/manifest.tsv"
    let page = ({ material: "Material", cyberpunk: "Cyberpunk", neobrutalism: "Neobrutalism", dodai: "Dodai" } | get $design)
    # the frames of the design's Components page, by name
    let ids = (open --raw scripts/design/frame-ids.js | str replace "PAGE" $"($page) / Components"
        | bunx @open-pencil/cli eval dist/design/dodai.fig --stdin | from json)
    $ids | items {|k, v| $"($k)\t($v.w)\t($v.h)" } | str join "\n" | save --force $"($out)/sizes.tsv"
    let theme = $design
    let pid = (with-env { DODAI_THEME: $theme, DODAI_SIZES: $"($out)/sizes.tsv", DODAI_PROBE: (if $probe { "1" } else { null }) } { job spawn { ^target/debug/examples/frames | save --force $manifest } })
    sleep $wait
    # the window by its id: it is captured even when something covers it
    let wid = (^osascript -l JavaScript scripts/window-id.js | str trim)
    if $wid == "" { error make { msg: "no window (Screen Recording permission?)" } }
    ^screencapture -x -o -l $wid $"($out)/shot.png"
    ^pkill -x frames | complete | ignore
    try { job kill $pid }
    let win = (open $manifest --raw | lines | first | split row "\t")
    let dims = (cargo run -q -p dodai-diff -- size $"($out)/shot.png" | split row " " | each { into float })
    let shot_w = $dims.0
    let shot_h = $dims.1
    let scale = ($shot_w / ($win.1 | into float))
    let top = ($shot_h / $scale - ($win.2 | into float))
    if $probe {
        cargo run -q -p dodai-diff -- probe $"($out)/shot.png" $manifest --scale ($scale | into string) --top $top
        return
    }
    for l in (open $manifest --raw | lines | skip 1) {
        let name = ($l | split row "\t" | first)
        let id = ($ids | get -o $name | get -o id)
        if $id != null {
            bunx @open-pencil/cli export dist/design/dodai.fig --node $id -f png --scale ($scale | into string) -o $"($out)/design/($name | str replace --all --regex '[/ ]' '_').png" | ignore
        }
    }
    cargo run -q -p dodai-diff -- grid $"($out)/shot.png" $manifest $"($out)/design" --scale ($scale | into string) --top $top --out $"($out)/diff" --max-blocks ($max_blocks | into string)
}
