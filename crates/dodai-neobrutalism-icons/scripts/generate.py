#!/usr/bin/env python3
"""Generate src/bold.rs from Phosphor Icons (MIT): the `bold` weight as the outline icon and
the `fill` weight as the `-fill` variant, renamed to Dodai/Material icon names.

usage: generate.py PHOSPHOR_CORE_CHECKOUT NAMES_FILE
NAMES_FILE lists the Dodai icon names (one per line). Names Phosphor spells the same way
are taken as they are; the rest come from names.py. Paths are rescaled from the 256 grid
to the `0 -960 960 960` box the Material pack uses, so both share one renderer.
"""
import re, sys, os
here = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, here)
from names import M

K = 960.0 / 256.0
ARGS = {'m': 2, 'l': 2, 'h': 1, 'v': 1, 'c': 6, 's': 4, 'q': 4, 't': 2, 'a': 7, 'z': 0}
STEPS = 8  # segments per cubic / per quarter turn of an arc

def num(x):
    t = f"{x:.1f}".rstrip('0').rstrip('.')
    return t if t not in ('-0', '') else '0'

def arc_points(x1, y1, rx, ry, phi, fa, fs, x2, y2):
    """SVG endpoint arc -> points along it (excluding the start)."""
    import math
    if rx == 0 or ry == 0:
        return [(x2, y2)]
    rx, ry = abs(rx), abs(ry)
    p = math.radians(phi)
    cp, sp = math.cos(p), math.sin(p)
    dx, dy = (x1 - x2) / 2, (y1 - y2) / 2
    x1p, y1p = cp * dx + sp * dy, -sp * dx + cp * dy
    lam = x1p**2 / rx**2 + y1p**2 / ry**2
    if lam > 1:
        r = math.sqrt(lam); rx *= r; ry *= r
    num_ = rx**2 * ry**2 - rx**2 * y1p**2 - ry**2 * x1p**2
    den = rx**2 * y1p**2 + ry**2 * x1p**2
    co = math.sqrt(max(0, num_ / den)) if den else 0
    if fa == fs:
        co = -co
    cxp, cyp = co * rx * y1p / ry, -co * ry * x1p / rx
    cx = cp * cxp - sp * cyp + (x1 + x2) / 2
    cy = sp * cxp + cp * cyp + (y1 + y2) / 2
    ang = lambda ux, uy, vx, vy: math.atan2(ux * vy - uy * vx, ux * vx + uy * vy)
    t1 = ang(1, 0, (x1p - cxp) / rx, (y1p - cyp) / ry)
    dt = ang((x1p - cxp) / rx, (y1p - cyp) / ry, (-x1p - cxp) / rx, (-y1p - cyp) / ry)
    if not fs and dt > 0: dt -= 2 * math.pi
    if fs and dt < 0: dt += 2 * math.pi
    n = max(2, int(abs(dt) / (math.pi / 2) * STEPS + 0.999))
    pts = []
    for i in range(1, n + 1):
        t = t1 + dt * i / n
        ex, ey = rx * math.cos(t), ry * math.sin(t)
        pts.append((cp * ex - sp * ey + cx, sp * ex + cp * ey + cy))
    pts[-1] = (x2, y2)
    return pts

def rescale(d):
    """Flatten a Phosphor path (any of MmLlHhVvCcSsQqTtAaZz) to M/L/Z on the 960 grid."""
    out, x, y, sx, sy = [], 0.0, 0.0, 0.0, 0.0
    last_c = None
    tr = lambda px, py: f"{num(px * K)},{num(py * K - 960)}"
    for m in re.finditer(r'([MmLlHhVvCcSsQqTtAaZz])([^MmLlHhVvCcSsQqTtAaZz]*)', d):
        c, body = m.group(1), m.group(2)
        v = [float(t) for t in re.findall(r'-?(?:\d+\.?\d*|\.\d+)(?:e-?\d+)?', body)]
        lc, rel = c.lower(), c.islower()
        n = ARGS[lc]
        if n == 0:
            out.append('Z'); x, y = sx, sy; last_c = None
            continue
        assert len(v) % n == 0, (c, body)
        first = True
        for i in range(0, len(v), n):
            a = v[i:i + n]
            ox, oy = (x, y) if rel else (0.0, 0.0)
            if lc == 'm':
                x, y = ox + a[0], oy + a[1]
                if first:
                    out.append('M' + tr(x, y)); sx, sy = x, y
                else:
                    out.append('L' + tr(x, y))
                last_c = None
            elif lc == 'l':
                x, y = ox + a[0], oy + a[1]; out.append('L' + tr(x, y)); last_c = None
            elif lc == 'h':
                x = ox + a[0]; out.append('L' + tr(x, y)); last_c = None
            elif lc == 'v':
                y = oy + a[0]; out.append('L' + tr(x, y)); last_c = None
            elif lc in 'cs':
                if lc == 'c':
                    p1 = (ox + a[0], oy + a[1]); p2 = (ox + a[2], oy + a[3]); p3 = (ox + a[4], oy + a[5])
                else:
                    p1 = (2 * x - last_c[0], 2 * y - last_c[1]) if last_c else (x, y)
                    p2 = (ox + a[0], oy + a[1]); p3 = (ox + a[2], oy + a[3])
                for k in range(1, STEPS + 1):
                    t = k / STEPS; u = 1 - t
                    px = u**3 * x + 3 * u * u * t * p1[0] + 3 * u * t * t * p2[0] + t**3 * p3[0]
                    py = u**3 * y + 3 * u * u * t * p1[1] + 3 * u * t * t * p2[1] + t**3 * p3[1]
                    out.append('L' + tr(px, py))
                x, y = p3; last_c = p2
            elif lc in 'qt':
                if lc == 'q':
                    q = (ox + a[0], oy + a[1]); e = (ox + a[2], oy + a[3])
                else:
                    q = (2 * x - last_c[0], 2 * y - last_c[1]) if last_c else (x, y)
                    e = (ox + a[0], oy + a[1])
                for k in range(1, STEPS + 1):
                    t = k / STEPS; u = 1 - t
                    out.append('L' + tr(u * u * x + 2 * u * t * q[0] + t * t * e[0],
                                        u * u * y + 2 * u * t * q[1] + t * t * e[1]))
                x, y = e; last_c = q
            elif lc == 'a':
                e = (ox + a[5], oy + a[6])
                for px, py in arc_points(x, y, a[0], a[1], a[2], int(a[3]), int(a[4]), e[0], e[1]):
                    out.append('L' + tr(px, py))
                x, y = e; last_c = None
            first = False
    return ''.join(out)

def path_of(file):
    svg = open(file).read()
    ds = re.findall(r'<path[^>]* d="([^"]+)"', svg)
    return ' '.join(rescale(d) for d in ds)

def main():
    core, names_file = sys.argv[1], sys.argv[2]
    names = sorted(set(l.strip() for l in open(names_file) if l.strip()))
    have = set(f[:-9] for f in os.listdir(f'{core}/assets/bold'))
    rows, missing = [], []
    for n in names:
        p = M.get(n) or n.replace('_', '-')
        if p not in have:
            missing.append(n)
            continue
        rows.append((n, path_of(f'{core}/assets/bold/{p}-bold.svg')))
        f = f'{core}/assets/fill/{p}-fill.svg'
        if os.path.exists(f):
            rows.append((n + '-fill', path_of(f)))
    rows.sort()
    with open(os.path.join(here, '..', 'src', 'bold.rs'), 'w') as o:
        o.write('// @generated by scripts/generate.py from Phosphor Icons (MIT). Do not edit.\n')
        o.write('use dodai_material_icons::Icon;\n\n')
        o.write(f'/// All {len(rows)} icons (outline + `-fill`), sorted by name.\n')
        o.write('pub static ALL: &[Icon] = &[\n')
        for n, p in rows:
            o.write(f'    Icon {{ name: "{n}", path: "{p}" }},\n')
        o.write('];\n')
    print(len(rows), 'icons; missing:', missing)

main()
