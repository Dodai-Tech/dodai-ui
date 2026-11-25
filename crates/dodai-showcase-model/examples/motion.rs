//! Writes the motion spec for the design file: the duration scale, the named easing curves
//! sampled, every entry effect at five points, and the overscroll feels:
//! `cargo run -p dodai-showcase-model --example motion -- dist/design`.
//!
//! It is the same data every toolkit animates from ([`dodai_core::motion`]), so the Motion page
//! of the design file cannot drift from the code.

use std::fmt::Write as _;
use std::path::PathBuf;

use dodai_core::motion::{material_easing as m, DurationToken, Easing, EntryKind, EntrySet};
use dodai_core::overscroll::{Overscroll, OverscrollFeel};

fn curve(name: &str, group: &str, e: Easing) -> String {
    let pts: Vec<String> = (0..=32)
        .map(|i| format!("{:.4}", e.apply(i as f32 / 32.0)))
        .collect();
    format!(
        "{{\"name\":\"{name}\",\"group\":\"{group}\",\"css\":\"{}\",\"points\":[{}]}}",
        e.to_css().replace('"', "'"),
        pts.join(",")
    )
}

fn feel(name: &str, f: OverscrollFeel) -> String {
    // Pull to the stop, let go, and sample the spring back every 1/60 s.
    let mut o = Overscroll::new(f);
    let t0 = std::time::Instant::now();
    for _ in 0..40 {
        o.on_wheel(40.0, true, false, t0);
    }
    let mut now = t0 + std::time::Duration::from_millis(100);
    let mut pts = vec![o.offset()];
    for _ in 0..60 {
        o.settle(now, 1.0 / 60.0);
        now += std::time::Duration::from_millis(16);
        pts.push(o.offset());
    }
    let pts: Vec<String> = pts.iter().map(|v| format!("{v:.2}")).collect();
    format!(
        "{{\"name\":\"{name}\",\"max\":{},\"stiffness\":{},\"damping\":{},\"release\":[{}]}}",
        f.max,
        f.stiffness,
        f.damping,
        pts.join(",")
    )
}

fn main() {
    let out = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "dist/design".into()),
    );
    std::fs::create_dir_all(&out).expect("create output dir");

    let durations: Vec<String> = DurationToken::ALL
        .iter()
        .map(|d| format!("{{\"name\":\"{}\",\"ms\":{}}}", d.name(), d.millis()))
        .collect();

    let easings = [
        curve("linear", "basic", Easing::Linear),
        curve("ease-in", "basic", Easing::EaseIn),
        curve("ease-out", "basic", Easing::EaseOut),
        curve("ease-in-out", "basic", Easing::EaseInOut),
        curve("standard", "material", m::STANDARD),
        curve("standard-decelerate", "material", m::STANDARD_DECELERATE),
        curve("standard-accelerate", "material", m::STANDARD_ACCELERATE),
        curve("emphasized", "material", m::EMPHASIZED),
        curve(
            "emphasized-decelerate",
            "material",
            m::EMPHASIZED_DECELERATE,
        ),
        curve(
            "emphasized-accelerate",
            "material",
            m::EMPHASIZED_ACCELERATE,
        ),
        curve("spring", "physical", Easing::SPRING),
        curve("spring-stiff", "physical", Easing::SPRING_STIFF),
        curve("spring-wobbly", "physical", Easing::SPRING_WOBBLY),
        curve("spring-gentle", "physical", Easing::SPRING_GENTLE),
        curve("bounce", "physical", Easing::Bounce),
        curve("elastic", "physical", Easing::Elastic),
        curve("back", "physical", Easing::Back(1.7)),
        curve("anticipate", "physical", Easing::Anticipate(1.7)),
    ];

    let mut entries = Vec::new();
    for k in EntryKind::ALL {
        if k == EntryKind::None {
            continue;
        }
        let set = EntrySet::of(k);
        let mut frames = String::new();
        for (i, p) in [0.0f32, 0.25, 0.5, 0.75, 1.0].into_iter().enumerate() {
            let (o, dx, dy) = set.frame(p, std::time::Duration::from_millis(220));
            if i > 0 {
                frames.push(',');
            }
            let _ = write!(frames, "[{o:.3},{dx:.2},{dy:.2}]");
        }
        entries.push(format!(
            "{{\"name\":\"{}\",\"frames\":[{frames}]}}",
            k.label()
        ));
    }

    let feels = [
        feel("soft (rounded designs)", OverscrollFeel::SOFT),
        feel("snappy (square designs)", OverscrollFeel::SNAPPY),
    ];

    let json = format!(
        "{{\"durations\":[{}],\"easings\":[{}],\"entries\":[{}],\"overscroll\":[{}]}}\n",
        durations.join(","),
        easings.join(","),
        entries.join(","),
        feels.join(",")
    );
    std::fs::write(out.join("motion.json"), json).expect("write motion.json");
    println!("wrote {}", out.join("motion.json").display());
}
