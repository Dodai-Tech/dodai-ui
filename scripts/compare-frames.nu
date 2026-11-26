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
    let theme = $design
    let pid = (with-env { DODAI_THEME: $theme } { job spawn { ^target/debug/examples/frames | save --force $manifest } })
    sleep $wait
    ^osascript -e 'tell application "System Events" to set frontmost of process "frames" to true' | complete | ignore
    sleep 1sec
    let r = (^osascript -e 'tell application "System Events" to tell process "frames" to get {position, size} of window 1' | complete)
    if $r.exit_code != 0 { error make { msg: "no window (Screen Recording / Accessibility permission?)" } }
    let n = ($r.stdout | str trim | split row ", " | each { into int })
    ^screencapture -x -R $"($n.0),($n.1),($n.2),($n.3)" $"($out)/shot.png"
    ^pkill -x frames | complete | ignore
    try { job kill $pid }
    let win = (open $manifest --raw | lines | first | split row "\t")
    let top = ($n.3 - ($win.2 | into float | math round))
    for l in (open $manifest --raw | lines | skip 1) {
        let name = ($l | split row "\t" | first)
        let id = ($ids | get -o $name)
        if $id != null {
            bunx @open-pencil/cli export dist/design/dodai.fig --node $id -f png --scale 2 -o $"($out)/design/($name).png" | ignore
        }
    }
    cargo run -q -p dodai-diff -- grid $"($out)/shot.png" $manifest $"($out)/design" --scale 2 --top $top --out $"($out)/diff" --max-blocks ($max_blocks | into string)
}
