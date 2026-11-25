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

fn main() -> ExitCode {
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
