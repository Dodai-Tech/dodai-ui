#!/usr/bin/env nu
# Screenshots the gpui showcase, one PNG per section, for `scripts/compare.nu`.
#
#   nu scripts/capture-gpui.nu [--only <substring>] [--size 900x700] [--out dist/compare/gpui]
#
# macOS only: needs a screen and the Screen Recording permission for your terminal. Opens the
# showcase on one section (`DODAI_ONLY`), reads the window's rectangle through System Events and
# captures that rectangle. The title of the section is the file name (non-alphanumerics -> `_`).
def main [
    --only: string = ""
    --size: string = "900x700"
    --out: path = "dist/compare/gpui"
    --wait: duration = 5sec
] {
    mkdir $out
    cargo build -q -p dodai-showcase
    let bin = "target/debug/gpui_showcase"
    let titles = (with-env { DODAI_LIST: "1" } { ^$bin | complete | get stderr }
        | lines | split column "\t" cat title | get title | where { $in | str contains $only })
    for t in $titles {
        let name = ($t | str replace --all --regex '[^A-Za-z0-9]+' '_' | str trim --char '_')
        let pid = (with-env { DODAI_ONLY: $t, DODAI_SIZE: $size } { job spawn { ^$bin | ignore } })
        sleep $wait
        # by window id, so a window on top of it cannot end up in the picture
        let wid = (with-env { WINDOW_OWNER: "gpui_showcase" } { ^osascript -l JavaScript scripts/window-id.js } | str trim)
        if $wid == "" {
            print $"skip ($t): no window"
        } else {
            ^screencapture -x -o -l $wid ($out | path join $"($name).png")
        }
        ^pkill gpui_showcase | complete | ignore
        try { job kill $pid }
    }
}
