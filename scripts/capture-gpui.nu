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
        let r = (^osascript -e 'tell application "System Events" to tell process "gpui_showcase" to get {position, size} of window 1' | complete)
        if $r.exit_code == 0 {
            let n = ($r.stdout | str trim | split row ", " | each { into int })
            ^screencapture -x -R $"($n.0),($n.1),($n.2),($n.3)" ($out | path join $"($name).png")
        } else {
            print $"skip ($t): no window"
        }
        ^pkill gpui_showcase | complete | ignore
        try { job kill $pid }
    }
}
