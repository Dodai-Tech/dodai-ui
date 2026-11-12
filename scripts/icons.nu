#!/usr/bin/env nu
# Derive every platform icon of an app from ONE master SVG.
#
#   nu <skill>/scripts/icons.nu [--id <app-id>] [--gui-assets <dir>]
#
# Master:  assets/icons/<id>.svg  (1024 × 1024, see references/design.md).
# Its marked blocks (<!-- detail:start --> … :end, likewise construction and rim) let
# the variants drop fine parts:
#   full         the master as drawn (Linux, Windows, ≥ 48 px)
#   small        no detail/construction/rim, the optional mark block enlarged: crisp at
#                16–32 px where fine lines only blur
#   macos        the full art on Apple's 1024 grid: 824 px body, margin and shadow
#   macos-small  the same around the small variant
#
# Writes (commit all of them; names match create-rust-project's package_desktop.nu):
#   assets/icons/<id>-<n>.png          Linux hicolor sizes (SIZES)
#   packaging/macos/<id>.icns          the .app icon, 16 – 1024 px (built here in pure Nushell:
#                                      no Mac, no iconutil; only needed if a .app is shipped)
#   packaging/windows/<id>.ico         installer and Start-menu icon
#   <gui-assets>/<id>.ico              embedded into <id>.exe (winres)
#   <gui-assets>/<id>-128.rgba         the window icon (Windows, X11): raw 8-bit RGBA
#   <gui-assets>/<id>-macos-512.png    the Dock icon outside a .app
#
# Needs a rasteriser (resvg or rsvg-convert) and ImageMagick (`magick`), on any OS.

export const SIZES = [16 22 24 32 48 64 96 128 256 512]
# The largest pixel size that gets a "small" variant.
export const SMALL_MAX = 32
# Sizes inside the Windows .ico: the shell picks 16–48, high-DPI up to 256.
export const ICO_SIZES = [16 20 24 32 40 48 64 96 128 256]
export const WINDOW_ICON_SIZE = 128
# Blocks of the master that the small variants drop.
export const DROPPED = [detail construction rim]
# Apple's macOS icon grid: the body is 824 of 1024 px, centred.
const MAC_BODY = 824

# A block of the master between its <!-- name:start --> and <!-- name:end --> markers.
def block-regex [name: string]: nothing -> string {
    $'\s*<!-- ($name):start -->.*?<!-- ($name):end -->'
}

# The inside of an <svg> document: everything between its opening and closing tag.
export def svg-body [svg: string]: nothing -> string {
    $svg | str replace --regex '(?s)^.*?<svg[^>]*>' '' | str replace --regex '(?s)</svg>\s*$' ''
}

# Wrap a 1024 px icon onto Apple's grid: scaled to the body size, centred, with
# the soft drop shadow macOS icons carry.
export def on-mac-grid [svg: string]: nothing -> string {
    let scale = $MAC_BODY / 1024
    let offset = (1024 - $MAC_BODY) / 2
    '<svg xmlns="http://www.w3.org/2000/svg" width="1024" height="1024" viewBox="0 0 1024 1024">
  <defs>
    <filter id="mac-shadow" x="-10%" y="-10%" width="120%" height="125%">
      <feDropShadow dx="0" dy="12" stdDeviation="14" flood-color="#000000" flood-opacity="0.35"/>
    </filter>
  </defs>
  <g filter="url(#mac-shadow)">
    <g transform="translate(OFFSET OFFSET) scale(SCALE)">BODY</g>
  </g>
</svg>
'
    | str replace --all 'OFFSET' ($offset | into string)
    | str replace 'SCALE' ($scale | into string)
    | str replace 'BODY' (svg-body $svg)
}

# The SVG of one variant of the master (see the header).
export def variant [master: string, kind: string]: nothing -> string {
    match $kind {
        "full" => $master
        "small" => {
            $DROPPED
            | reduce --fold $master {|name, svg| $svg | str replace --all --regex $"\(?s\)(block-regex $name)" '' }
            | str replace --regex '<!-- mark:start -->' '<!-- mark:start --><g transform="translate(512 512) scale(1.12) translate(-512 -512)">'
            | str replace --regex '<!-- mark:end -->' '</g><!-- mark:end -->'
        }
        "macos" => (on-mac-grid $master)
        "macos-small" => (on-mac-grid (variant $master small))
        _ => { error make {msg: $"unknown icon variant: ($kind)"} }
    }
}

# Which variant a raster of `px` pixels uses, on the Mac grid or not.
export def kind-for [px: int, mac: bool]: nothing -> string {
    match [$mac ($px <= $SMALL_MAX)] {
        [true true] => "macos-small"
        [true false] => "macos"
        [false true] => "small"
        _ => "full"
    }
}

# The PNG entries of an .icns: OSType and pixel size, 16 px to 1024 px (retina pairs included).
export const ICNS_TYPES = [
    {ostype: icp4, px: 16} {ostype: icp5, px: 32} {ostype: icp6, px: 64}
    {ostype: ic07, px: 128} {ostype: ic08, px: 256} {ostype: ic09, px: 512} {ostype: ic10, px: 1024}
    {ostype: ic11, px: 32} {ostype: ic12, px: 64} {ostype: ic13, px: 256} {ostype: ic14, px: 512}
]

# A 32-bit big-endian integer as 4 bytes.
export def u32-be [n: int]: nothing -> binary {
    [24 16 8 0] | each {|shift| ($n // (2 ** $shift)) mod 256 | into binary | bytes at 0..0 } | bytes collect
}

# One .icns entry: its OSType, its length (header included), then the PNG bytes.
export def icns-entry [ostype: string, png: binary]: nothing -> binary {
    [($ostype | into binary) (u32-be (($png | bytes length) + 8)) $png] | bytes collect
}

# An .icns file: the `icns` magic, the total length, then the entries.
export def icns-bytes [entries: list<binary>]: nothing -> binary {
    let body = $entries | bytes collect
    [("icns" | into binary) (u32-be (($body | bytes length) + 8)) $body] | bytes collect
}

# The app id from the lone assets/icons/*.svg, or null when there is not exactly one.
export def detect-id [svgs: list<string>]: nothing -> any {
    if ($svgs | length) == 1 { $svgs.0 | path parse | get stem } else { null }
}

# Where the window/Dock assets go: the first crates/*gui*/assets, else assets/app.
export def gui-assets-dir [candidates: list<string>]: nothing -> string {
    if ($candidates | is-empty) { "assets/app" } else { $candidates | sort | first }
}

# Every file `main` writes, relative to the repository root.
export def outputs [id: string, gui: string]: nothing -> list<string> {
    let pngs = $SIZES | each {|n| $"assets/icons/($id)-($n).png" }
    $pngs ++ [
        $"packaging/macos/($id).icns"
        $"packaging/windows/($id).ico"
        $"($gui)/($id).ico"
        $"($gui)/($id)-($WINDOW_ICON_SIZE).rgba"
        $"($gui)/($id)-macos-512.png"
    ]
}

# The rasteriser's argv for svg -> png at px × px, whichever tool is installed.
export def raster-args [tool: string, svg: string, px: int, out: string]: nothing -> list<string> {
    match $tool {
        "resvg" => ["--width" ($px | into string) "--height" ($px | into string) $svg $out]
        "rsvg-convert" => ["-w" ($px | into string) "-h" ($px | into string) $svg "-o" $out]
        _ => { error make {msg: $"unknown rasteriser: ($tool)"} }
    }
}

# The first installed SVG rasteriser, or null.
export def pick-rasteriser [installed: list<string>]: nothing -> any {
    [resvg rsvg-convert] | where {|t| $t in $installed } | get 0?
}

def rasteriser []: nothing -> string {
    let found = pick-rasteriser (which resvg rsvg-convert | get command)
    if $found == null { error make {msg: "no SVG rasteriser: brew install resvg  /  apt install librsvg2-bin  /  cargo install resvg"} }
    $found
}

def need [tool: string, hint: string] {
    if (which $tool | is-empty) { error make {msg: $"($tool) not found: ($hint)"} }
}

def main [
    --id: string        # app id (default: the only assets/icons/*.svg)
    --gui-assets: path  # where the window/Dock assets go (default: crates/*gui*/assets)
] {
    let id = $id | default (detect-id (glob assets/icons/*.svg)) 
    if $id == null { error make {msg: "pass --id <app-id> (assets/icons/<id>.svg is the master)"} }
    let master_path = $"assets/icons/($id).svg"
    if not ($master_path | path exists) { error make {msg: $"($master_path) not found: draw the master first \(references/design.md\)"} }
    let gui = $gui_assets | default (gui-assets-dir (glob crates/*gui*/assets | each {|d| $d | path relative-to $env.PWD }))
    let raster = rasteriser
    need magick "brew install imagemagick  /  apt install imagemagick"

    let tmp = mktemp --directory
    let master = open --raw $master_path
    let variants = ["full" "small" "macos" "macos-small"] | reduce --fold {} {|k, acc|
        let path = $"($tmp)/($k).svg"
        variant $master $k | save --force $path
        $acc | insert $k $path
    }
    let render = {|kind: string, px: int, out: string|
        ^$raster ...(raster-args $raster ($variants | get $kind) $px $out)
    }

    # Linux hicolor PNGs
    for n in $SIZES { do $render (kind-for $n false) $n $"assets/icons/($id)-($n).png" }

    # Windows .ico: uncompressed BMP entries, which NSIS and every Windows version read
    let ico_pngs = $ICO_SIZES | each {|n|
        let p = $"($tmp)/ico-($n).png"
        do $render (kind-for $n false) $n $p
        $p
    }
    mkdir packaging/windows $gui
    ^magick ...$ico_pngs -define icon:format=bmp $"packaging/windows/($id).ico"
    cp $"packaging/windows/($id).ico" $"($gui)/($id).ico"

    # Window icon (Windows title bar and taskbar, X11) as raw RGBA, and the Dock
    # icon a bare binary sets on macOS
    let win_png = $"($tmp)/window.png"
    do $render (kind-for $WINDOW_ICON_SIZE false) $WINDOW_ICON_SIZE $win_png
    ^magick $win_png -depth 8 $"RGBA:($gui)/($id)-($WINDOW_ICON_SIZE).rgba"
    do $render macos 512 $"($gui)/($id)-macos-512.png"

    # macOS .icns, written by hand so no Mac (and no iconutil) is needed
    mkdir packaging/macos
    let entries = $ICNS_TYPES | each {|e|
        let p = $"($tmp)/icns-($e.ostype).png"
        do $render (kind-for $e.px true) $e.px $p
        icns-entry $e.ostype (open --raw $p)
    }
    icns-bytes $entries | save --force --raw $"packaging/macos/($id).icns"

    rm --recursive $tmp
    print $"(ansi green)✓(ansi reset) icons written:\n(outputs $id $gui | where {|f| $f | path exists } | each {|f| $'  ($f)' } | str join "\n")"
}
