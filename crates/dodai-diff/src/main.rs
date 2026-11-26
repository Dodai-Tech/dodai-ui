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
        let design = match Picture::open_on(&Path::new(dir).join(format!("{name}.png")), bg) {
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
            let _ = img.save(&o.join(format!("{name}.png")));
        }
        println!("{:>6.2}% px {:>6.2}% blocks  {name}", r.pixels, r.blocks);
        failed |= max.is_some_and(|m| r.blocks > m);
    }
    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn main() -> ExitCode {
    let all: Vec<String> = std::env::args().skip(1).collect();
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
