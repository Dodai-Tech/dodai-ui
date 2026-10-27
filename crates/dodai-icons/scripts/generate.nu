#!/usr/bin/env nu
# Generate src/bold.rs from Phosphor Icons (MIT): the `bold` weight is the outline icon and
# the `fill` weight is the `-fill` variant, renamed to Dodai / Material icon names.
#
#     git clone --depth 1 https://github.com/phosphor-icons/core /tmp/phosphor
#     nu scripts/generate.nu /tmp/phosphor names.txt
#
# NAMES_FILE lists the Dodai icon names, one per line. A name Phosphor spells the same way is
# taken as is; the rest come from names.nuon. Paths are rescaled from Phosphor's 256 grid to
# the `0 -960 960 960` box the Material pack uses, and curves are flattened to straight
# segments, so both packs share one renderer.

const K = 3.75  # 960 / 256
const STEPS = 8 # segments per cubic or quadratic, and per quarter turn of an arc
const PI = 3.141592653589793

def fmt [x: float] {
    let t = ($x | math round --precision 1 | into string | str replace --regex '\.0$' '')
    if $t == "-0" { "0" } else { $t }
}

# Path point on the 960 grid, as an `L` segment.
def pt [x: float, y: float] { $"L(fmt ($x * $K)),(fmt ($y * $K - 960.0))" }

def atan2 [y: float, x: float] {
    if $x > 0.0 {
        ($y / $x) | math arctan
    } else if $x < 0.0 and $y >= 0.0 {
        ($y / $x | math arctan) + $PI
    } else if $x < 0.0 {
        ($y / $x | math arctan) - $PI
    } else if $y > 0.0 {
        $PI / 2.0
    } else if $y < 0.0 {
        -1.0 * $PI / 2.0
    } else {
        0.0
    }
}

# SVG endpoint arc -> points along it (the start is excluded), as [x y] pairs.
def arc_points [x1: float, y1: float, rx0: float, ry0: float, phi: float, fa: int, fs: int, x2: float, y2: float] {
    if $rx0 == 0.0 or $ry0 == 0.0 { return [[$x2 $y2]] }
    let p = ($phi * $PI / 180.0)
    let cp = ($p | math cos)
    let sp = ($p | math sin)
    let dx = (($x1 - $x2) / 2.0)
    let dy = (($y1 - $y2) / 2.0)
    let x1p = ($cp * $dx + $sp * $dy)
    let y1p = (-1.0 * $sp * $dx + $cp * $dy)
    mut rx = ($rx0 | math abs)
    mut ry = ($ry0 | math abs)
    let lam = (($x1p * $x1p) / ($rx * $rx) + ($y1p * $y1p) / ($ry * $ry))
    if $lam > 1.0 {
        let r = ($lam | math sqrt)
        $rx = $rx * $r
        $ry = $ry * $r
    }
    let num = ($rx * $rx * $ry * $ry - $rx * $rx * $y1p * $y1p - $ry * $ry * $x1p * $x1p)
    let den = ($rx * $rx * $y1p * $y1p + $ry * $ry * $x1p * $x1p)
    mut co = (if $den != 0.0 { [0.0 ($num / $den)] | math max | math sqrt } else { 0.0 })
    if $fa == $fs { $co = -1.0 * $co }
    let cxp = ($co * $rx * $y1p / $ry)
    let cyp = (-1.0 * $co * $ry * $x1p / $rx)
    let cx = ($cp * $cxp - $sp * $cyp + ($x1 + $x2) / 2.0)
    let cy = ($sp * $cxp + $cp * $cyp + ($y1 + $y2) / 2.0)
    let ux = (($x1p - $cxp) / $rx)
    let uy = (($y1p - $cyp) / $ry)
    let vx = ((-1.0 * $x1p - $cxp) / $rx)
    let vy = ((-1.0 * $y1p - $cyp) / $ry)
    let t1 = (atan2 $uy $ux)
    mut dt = (atan2 ($ux * $vy - $uy * $vx) ($ux * $vx + $uy * $vy))
    if $fs == 0 and $dt > 0.0 { $dt = $dt - 2.0 * $PI }
    if $fs == 1 and $dt < 0.0 { $dt = $dt + 2.0 * $PI }
    let n = ([2 ((($dt | math abs) / ($PI / 2.0) * $STEPS + 0.999) | into int)] | math max)
    mut pts = []
    for i in 1..$n {
        let t = ($t1 + $dt * $i / $n)
        let ex = ($rx * ($t | math cos))
        let ey = ($ry * ($t | math sin))
        $pts = ($pts | append [[($cp * $ex - $sp * $ey + $cx) ($sp * $ex + $cp * $ey + $cy)]])
    }
    $pts | update (($pts | length) - 1) [$x2 $y2]
}

const ARGS = {m: 2, l: 2, h: 1, v: 1, c: 6, s: 4, q: 4, t: 2, a: 7}

# Flatten a Phosphor path (any of MmLlHhVvCcSsQqTtAaZz) to M/L/Z on the 960 grid.
def rescale [d: string] {
    mut out = []
    mut x = 0.0
    mut y = 0.0
    mut sx = 0.0
    mut sy = 0.0
    mut last = [] # last control point, for S / T reflection
    let cmds = ($d | parse --regex '(?P<c>[MmLlHhVvCcSsQqTtAaZz])(?P<body>[^MmLlHhVvCcSsQqTtAaZz]*)')
    for cmd in $cmds {
        let lc = ($cmd.c | str lowercase)
        let rel = ($cmd.c != ($cmd.c | str uppercase))
        if $lc == "z" {
            $out = ($out | append "Z")
            $x = $sx
            $y = $sy
            $last = []
            continue
        }
        let v = ($cmd.body | parse --regex '(?P<n>-?(?:\d+\.?\d*|\.\d+)(?:e-?\d+)?)' | get n | into float)
        let n = ($ARGS | get $lc)
        if ($v | length) mod $n != 0 { error make { msg: $"bad arg count for ($cmd.c): ($cmd.body)" } }
        mut first = true
        for i in 0..<(($v | length) // $n) {
            let a = ($v | skip ($i * $n) | first $n)
            let ox = (if $rel { $x } else { 0.0 })
            let oy = (if $rel { $y } else { 0.0 })
            match $lc {
                "m" => {
                    $x = $ox + $a.0
                    $y = $oy + $a.1
                    if $first { $out = ($out | append $"M(fmt ($x * $K)),(fmt ($y * $K - 960.0))"); $sx = $x; $sy = $y } else { $out = ($out | append (pt $x $y)) }
                    $last = []
                }
                "l" => { $x = $ox + $a.0; $y = $oy + $a.1; $out = ($out | append (pt $x $y)); $last = [] }
                "h" => { $x = $ox + $a.0; $out = ($out | append (pt $x $y)); $last = [] }
                "v" => { $y = $oy + $a.0; $out = ($out | append (pt $x $y)); $last = [] }
                "c" | "s" => {
                    let p1 = (if $lc == "c" { [($ox + $a.0) ($oy + $a.1)] } else if not ($last | is-empty) { [(2.0 * $x - $last.0) (2.0 * $y - $last.1)] } else { [$x $y] })
                    let k = (if $lc == "c" { 2 } else { 0 })
                    let p2 = [($ox + ($a | get $k)) ($oy + ($a | get ($k + 1)))]
                    let p3 = [($ox + ($a | get ($k + 2))) ($oy + ($a | get ($k + 3)))]
                    for s in 1..$STEPS {
                        let t = ($s / $STEPS)
                        let u = (1.0 - $t)
                        let px = ($u * $u * $u * $x + 3.0 * $u * $u * $t * $p1.0 + 3.0 * $u * $t * $t * $p2.0 + $t * $t * $t * $p3.0)
                        let py = ($u * $u * $u * $y + 3.0 * $u * $u * $t * $p1.1 + 3.0 * $u * $t * $t * $p2.1 + $t * $t * $t * $p3.1)
                        $out = ($out | append (pt $px $py))
                    }
                    $x = $p3.0
                    $y = $p3.1
                    $last = $p2
                }
                "q" | "t" => {
                    let q = (if $lc == "q" { [($ox + $a.0) ($oy + $a.1)] } else if not ($last | is-empty) { [(2.0 * $x - $last.0) (2.0 * $y - $last.1)] } else { [$x $y] })
                    let k = (if $lc == "q" { 2 } else { 0 })
                    let e = [($ox + ($a | get $k)) ($oy + ($a | get ($k + 1)))]
                    for s in 1..$STEPS {
                        let t = ($s / $STEPS)
                        let u = (1.0 - $t)
                        $out = ($out | append (pt ($u * $u * $x + 2.0 * $u * $t * $q.0 + $t * $t * $e.0) ($u * $u * $y + 2.0 * $u * $t * $q.1 + $t * $t * $e.1)))
                    }
                    $x = $e.0
                    $y = $e.1
                    $last = $q
                }
                "a" => {
                    let e = [($ox + $a.5) ($oy + $a.6)]
                    for q in (arc_points $x $y $a.0 $a.1 $a.2 ($a.3 | into int) ($a.4 | into int) $e.0 $e.1) {
                        $out = ($out | append (pt $q.0 $q.1))
                    }
                    $x = $e.0
                    $y = $e.1
                    $last = []
                }
            }
            $first = false
        }
    }
    $out | str join
}

def path_of [file: string] {
    open --raw $file | parse --regex '<path[^>]* d="(?P<d>[^"]+)"' | get d | each {|d| rescale $d } | str join " "
}

use ../../../scripts/icon-names.nu pascal

def main [core: string, names_file: string] {
    let map = (open ($env.FILE_PWD | path join names.nuon))
    let names = (open --raw $names_file | lines | str trim | where $it != "" | uniq | sort)
    let have = (ls ($core | path join assets bold) | get name | each {|f| $f | path basename | str replace '-bold.svg' '' })
    mut rows = []
    mut missing = []
    for n in $names {
        let p = ($map | get -o $n | default ($n | str replace --all '_' '-'))
        if $p not-in $have { $missing = ($missing | append $n); continue }
        $rows = ($rows | append { name: $n, path: (path_of ($core | path join assets bold $"($p)-bold.svg")) })
        let f = ($core | path join assets fill $"($p)-fill.svg")
        if ($f | path exists) { $rows = ($rows | append { name: $"($n)-fill", path: (path_of $f) }) }
    }
    # Every other Phosphor icon, under Phosphor's own name. A Dodai name wins a clash.
    let dodai = ($rows | get name)
    let meta = (open --raw ($core | path join src icons.ts)
        | parse --regex '(?s)name: "(?P<n>[a-z0-9-]+)",\s*pascal_name:.*?figma_category: FigmaCategory\.(?P<c>[A-Z_]+),')
    mut native = []
    mut regular = []
    for m in $meta {
        let b = ($core | path join assets bold $"($m.n)-bold.svg")
        if not ($b | path exists) { continue }
        if $m.n not-in $dodai {
            $rows = ($rows | append { name: $m.n, path: (path_of $b) })
        }
        let f = ($core | path join assets fill $"($m.n)-fill.svg")
        if ($f | path exists) and $"($m.n)-fill" not-in $dodai {
            $rows = ($rows | append { name: $"($m.n)-fill", path: (path_of $f) })
        }
        $native = ($native | append { name: $m.n, cat: $m.c })
        # The lighter `regular` weight goes to the shared drawing source.
        let r = ($core | path join assets regular $"($m.n).svg")
        if ($r | path exists) { $regular = ($regular | append { name: $m.n, path: (path_of $r) }) }
        if ($f | path exists) { $regular = ($regular | append { name: $"($m.n)-fill", path: (path_of $f) }) }
    }
    let sorted = ($rows | sort-by name | uniq-by name)
    let body = ($sorted | each {|r| $"    Icon { name: \"($r.name)\", path: \"($r.path)\" },\n" } | str join)
    [
        "// @generated by scripts/generate.nu from Phosphor Icons (MIT). Do not edit.\n"
        "use crate::material::Icon;\n\n"
        $"/// All ($sorted | length) icons \(outline + `-fill`\), sorted by name.\n"
        "pub static ALL: &[Icon] = &[\n"
        $body
        "];\n"
    ] | str join | save --force ($env.FILE_PWD | path dirname | path join src bold_data.rs)
    let groups = ($native | group-by cat | transpose cat items | sort-by cat)
    let cats = ($groups | each {|g|
        let label = ($g.cat | str replace --all '_' ' ' | str lowercase)
        let names = ($g.items | get name | sort | each {|n| $"IconId::(pascal $n)" } | str join ", ")
        $"    \(\"($label)\", &[($names)]\),\n"
    } | str join)
    [
        "// @generated by scripts/generate.nu from Phosphor Icons (MIT). Do not edit.\n\n"
        "use dodai_icon_id::IconId;\n\n"
        "/// Phosphor's own icon categories (each icon also has a filled form).\n"
        "pub static CATEGORIES: &[(&str, &[IconId])] = &[\n"
        $cats
        "];\n"
    ] | str join | save --force ($env.FILE_PWD | path dirname | path join src phosphor categories.rs)
    let rbody = ($regular | sort-by name | each {|r| $"    Icon { name: \"($r.name)\", path: \"($r.path)\" },\n" } | str join)
    [
        "// @generated by dodai-icons/scripts/generate.nu from Phosphor Icons (MIT). Do not edit.\n"
        "use crate::material::Icon;\n\n"
        $"/// All ($regular | length) icons \(`regular` weight + `-fill`\), sorted by name.\n"
        "pub static ALL: &[Icon] = &[\n"
        $rbody
        "];\n"
    ] | str join | save --force ($env.FILE_PWD | path dirname | path join src phosphor regular.rs)
    print $"($sorted | length) icons; missing: ($missing)"
}
