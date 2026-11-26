//! `dodai-diff A B [--tolerance N] [--out DIFF.png]` compares two PNGs; with two directories it
//! compares the files they share by name and prints a table (JSON lines with `--json`).
//! `--max-blocks P` makes the exit code 1 when any pair differs by more than P percent of cells
//! or has the wrong size; files in only one directory are listed and fail too.

use dodai_diff::{compare, Options, Picture, Report};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn pngs(dir: &Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| n.ends_with(".png"))
        .collect();
    v.sort();
    v
}

fn run(a: &Path, b: &Path, opt: Options, out: Option<&Path>) -> Result<Report, dodai_diff::Error> {
    let (r, img) = compare(&Picture::open(a)?, &Picture::open(b)?, opt);
    if let Some(o) = out {
        img.save(o)?;
    }
    Ok(r)
}

/// `dodai-diff grid SHOT MANIFEST DESIGN_DIR --scale S --top T [--out DIR] [--max-blocks P]`:
/// the screenshot holds a grid of components (`MANIFEST`: `window W H`, then `name X Y` in points,
/// tab separated); each is cropped to the size of the design export of the same name and compared.
fn grid(rest: &[String]) -> ExitCode {
    let (mut pos, mut scale, mut top, mut out, mut max) =
        (Vec::new(), 1.0f32, 0.0f32, None::<PathBuf>, None::<f32>);
    let mut it = rest.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--scale" => scale = it.next().and_then(|v| v.parse().ok()).unwrap_or(scale),
            "--top" => top = it.next().and_then(|v| v.parse().ok()).unwrap_or(top),
            "--out" => out = it.next().map(PathBuf::from),
            "--max-blocks" => max = it.next().and_then(|v| v.parse().ok()),
            _ => pos.push(a.clone()),
        }
    }
    let [shot, manifest, dir] = pos.as_slice() else {
        eprintln!("usage: dodai-diff grid SHOT MANIFEST DESIGN_DIR --scale S --top T [--out DIR] [--max-blocks P]");
        return ExitCode::from(2);
    };
    let shot = match Picture::open(Path::new(shot)) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(2);
        }
    };
    if let Some(o) = &out {
        let _ = std::fs::create_dir_all(o);
    }
    let text = std::fs::read_to_string(manifest).unwrap_or_default();
    let mut failed = false;
    for line in text.lines().filter(|l| !l.starts_with("window")) {
        let f: Vec<&str> = line.split('\t').collect();
        let [name, x, y] = f.as_slice() else { continue };
        let file = name.replace(['/', ' '], "_");
        let (Ok(x), Ok(y)) = (x.parse::<f32>(), y.parse::<f32>()) else {
            continue;
        };
        // the design's frames are transparent: lay them on the colour around the component
        let bg = shot
            .at(
                ((x - 4.0) * scale) as usize,
                ((y + top - 4.0) * scale) as usize,
            )
            .unwrap_or([255, 255, 255]);
        let design = match Picture::open_on(&Path::new(dir).join(format!("{file}.png")), bg) {
            Ok(p) => p,
            Err(_) => {
                println!("   no design export: {name}");
                failed = true;
                continue;
            }
        };
        let mine = shot.crop(
            (x * scale) as usize,
            ((y + top) * scale) as usize,
            design.width,
            design.height,
        );
        let (r, img) = compare(&mine, &design, Options::default());
        if let Some(o) = &out {
            let _ = img.save(&o.join(format!("{file}.png")));
        }
        let fmt = |b: Option<(usize, usize, usize, usize)>| {
            b.map_or("empty".into(), |(x, y, w, h)| format!("{w}x{h}@{x},{y}"))
        };
        println!(
            "{:>6.2}% px {:>6.2}% blocks  {name}  gpui {}  design {}",
            r.pixels,
            r.blocks,
            fmt(mine.content(bg, 6)),
            fmt(design.content(bg, 6)),
        );
        failed |= max.is_some_and(|m| r.blocks > m);
    }
    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

/// `dodai-diff size FILE` prints `W H` in pixels.
fn size(rest: &[String]) -> ExitCode {
    match rest.first().map(|p| Picture::open(Path::new(p))) {
        Some(Ok(p)) => {
            println!("{} {}", p.width, p.height);
            ExitCode::SUCCESS
        }
        Some(Err(e)) => {
            eprintln!("{e}");
            ExitCode::from(2)
        }
        None => {
            eprintln!("usage: dodai-diff size FILE");
            ExitCode::from(2)
        }
    }
}

/// Magenta, loosely: the display's colour profile shifts what the screenshot holds.
fn is_probe(p: Option<[u8; 3]>) -> bool {
    p.is_some_and(|[r, g, b]| r > 200 && b > 200 && g < 130)
}

/// `dodai-diff probe SHOT MANIFEST --scale S --top T`: the screenshot was taken with
/// `DODAI_PROBE=1`, so each cell holds a 16pt magenta square at its origin. Checks that every
/// square sits exactly where the manifest says (so the crop offsets of `grid` are right).
fn probe(rest: &[String]) -> ExitCode {
    let (mut pos, mut scale, mut top) = (Vec::new(), 1.0f32, 0.0f32);
    let mut it = rest.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--scale" => scale = it.next().and_then(|v| v.parse().ok()).unwrap_or(scale),
            "--top" => top = it.next().and_then(|v| v.parse().ok()).unwrap_or(top),
            _ => pos.push(a.clone()),
        }
    }
    let [shot, manifest] = pos.as_slice() else {
        eprintln!("usage: dodai-diff probe SHOT MANIFEST --scale S --top T");
        return ExitCode::from(2);
    };
    let Ok(shot) = Picture::open(Path::new(shot)) else {
        eprintln!("cannot read {shot}");
        return ExitCode::from(2);
    };
    let text = std::fs::read_to_string(manifest).unwrap_or_default();
    let (mut bad, mut n) = (0, 0);
    for line in text.lines().filter(|l| !l.starts_with("window")) {
        let f: Vec<&str> = line.split('\t').collect();
        let [name, x, y] = f.as_slice() else { continue };
        let (Ok(x), Ok(y)) = (x.parse::<f32>(), y.parse::<f32>()) else {
            continue;
        };
        let (px, py) = ((x * scale) as usize, ((y + top) * scale) as usize);
        let side = (16.0 * scale) as usize;
        let inside = is_probe(shot.at(px, py)) && is_probe(shot.at(px + side - 1, py + side - 1));
        let outside = !is_probe(shot.at(px.wrapping_sub(2), py))
            && !is_probe(shot.at(px, py.wrapping_sub(2)))
            && !is_probe(shot.at(px + side + 1, py + side + 1));
        n += 1;
        if !(inside && outside) {
            bad += 1;
            println!("probe off: {name} at {x},{y}");
        }
    }
    let first = (0..shot.height)
        .flat_map(|y| (0..shot.width).map(move |x| (x, y)))
        .find(|&(x, y)| is_probe(shot.at(x, y)));
    if let Some((x, y)) = first {
        println!("first square at pixel {x},{y}");
    }
    println!("probe: {} of {n} squares in place", n - bad);
    if bad > 0 || n == 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn main() -> ExitCode {
    let all: Vec<String> = std::env::args().skip(1).collect();
    match all.first().map(String::as_str) {
        Some("size") => return size(&all[1..]),
        Some("probe") => return probe(&all[1..]),
        _ => {}
    }
    if all.first().map(String::as_str) == Some("grid") {
        return grid(&all[1..]);
    }
    let mut args = std::env::args().skip(1);
    let (mut pos, mut opt, mut out, mut json, mut max) = (
        Vec::<PathBuf>::new(),
        Options::default(),
        None,
        false,
        None::<f32>,
    );
    while let Some(a) = args.next() {
        match a.as_str() {
            "--tolerance" => {
                opt.tolerance = args
                    .next()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(opt.tolerance)
            }
            "--out" => out = args.next().map(PathBuf::from),
            "--max-blocks" => max = args.next().and_then(|v| v.parse().ok()),
            "--json" => json = true,
            _ => pos.push(PathBuf::from(a)),
        }
    }
    let [a, b] = pos.as_slice() else {
        eprintln!("usage: dodai-diff A B [--tolerance N] [--out DIFF] [--max-blocks P] [--json]");
        return ExitCode::from(2);
    };
    let mut failed = false;
    let show = |name: &str, r: &Report| -> bool {
        if json {
            println!("{}", r.json(name));
        } else {
            let size = if r.same_size() {
                String::new()
            } else {
                format!("  size {:?} vs {:?}", r.a, r.b)
            };
            println!(
                "{:>6.2}% px {:>6.2}% blocks  {name}{size}",
                r.pixels, r.blocks
            );
        }
        max.is_some_and(|m| r.blocks > m || !r.same_size())
    };
    if a.is_dir() && b.is_dir() {
        if let Some(o) = &out {
            let _ = std::fs::create_dir_all(o);
        }
        let (la, lb) = (pngs(a), pngs(b));
        for n in &la {
            if !lb.contains(n) {
                println!("   only in {}: {n}", a.display());
                failed = true;
                continue;
            }
            match run(
                &a.join(n),
                &b.join(n),
                opt,
                out.as_ref().map(|o| o.join(n)).as_deref(),
            ) {
                Ok(r) => failed |= show(n, &r),
                Err(e) => {
                    eprintln!("{e}");
                    failed = true;
                }
            }
        }
        for n in lb.iter().filter(|n| !la.contains(n)) {
            println!("   only in {}: {n}", b.display());
            failed = true;
        }
    } else {
        match run(a, b, opt, out.as_deref()) {
            Ok(r) => failed |= show(&a.display().to_string(), &r),
            Err(e) => {
                eprintln!("{e}");
                return ExitCode::from(2);
            }
        }
    }
    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
