#!/usr/bin/env nu
# Screenshot one gpui_showcase category (macOS).
#
#   nu dshot.nu out.png CAT [ONLY] [THEME] [WAIT] [--size 980x760] [--bin ./target/debug/gpui_showcase]
#
# Launches the showcase, brings it to the front, finds its window id via
# CGWindowListCopyWindowInfo and captures that window with `screencapture -l`.

const REPO = "/Users/EBEMXC4/Projects/dodai-ui"

def kill_showcases [] {
    for p in [target/debug/gpui_showcase examples/gpui_showcase] { ^pkill -f $p | complete | ignore }
}

# Swift snippet printing the first on-screen window number owned by `pid`.
export def window_query [pid: int] {
    $"import CoreGraphics
for w in CGWindowListCopyWindowInfo\(.optionOnScreenOnly, kCGNullWindowID) as! [[String:Any]] { if let p = w[\"kCGWindowOwnerPID\"] as? Int, p == ($pid) { print\(w[\"kCGWindowNumber\"]!); break } }"
}

export def showcase_env [cat: string, only: string, theme: string, size: string] {
    { DODAI_SIZE: $size, DODAI_CAT: $cat, DODAI_THEME: $theme }
    | if ($only | is-empty) { $in } else { insert DODAI_ONLY $only }
}

def main [
    out: path, cat: string = "0", only: string = "", theme: string = "cyberpunk", wait: number = 5
    --size: string = "980x760"
    --bin: string = "./target/debug/gpui_showcase"
] {
    let out = $out | path expand
    kill_showcases
    sleep 400ms
    cd $REPO
    let job = with-env (showcase_env $cat $only $theme $size) { job spawn { try { ^$bin o+e> /dev/null } } }
    sleep ($wait * 1sec)
    # Exact process name: -f would also match e.g. `cargo build --bin gpui_showcase`.
    let pid = ^pgrep -n -x ($bin | path basename) | str trim | into int
    ^osascript -e $"tell application \"System Events\" to set frontmost of \(first process whose unix id is ($pid)) to true" | complete | ignore
    sleep 1500ms
    mut id = ""
    for _ in 1..6 {
        $id = (^swift -e (window_query $pid) | complete | get stdout | str trim)
        if ($id | is-not-empty) { break }
        sleep 2sec
    }
    if ($id | is-empty) { kill_showcases; error make --unspanned { msg: "no showcase window found (screen locked?)" } }
    ^screencapture -x -l $id $out
    kill_showcases
    try { job kill $job }
    ^sips -Z 1300 $out | complete | ignore
    print $out
}
