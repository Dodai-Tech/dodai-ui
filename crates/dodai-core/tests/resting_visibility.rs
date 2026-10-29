//! Controls at rest must be findable: every theme, on the page and on a card. Boundaries of
//! unchecked boxes, rings, off switches and empty fields need 3:1 (WCAG 1.4.11); an off
//! switch's handle must stand out from its track; a progress or slider rail must show.

use dodai_core::{Color, ControlState, Theme, ToggleKind};

/// `c` painted over the opaque `under`.
fn over(c: Color, under: Color) -> Color {
    under.mix(Color { a: 255, ..c }, c.a as f32 / 255.0)
}

fn ratio(c: Color, bg: Color) -> f32 {
    over(c, bg).contrast(bg)
}

fn failures(t: &Theme) -> Vec<String> {
    let p = &t.palette;
    let idle = ControlState::Idle;
    let mut out = Vec::new();
    let mut need = |what: &str, got: f32, min: f32| {
        if got < min {
            out.push(format!("{}: {what} {got:.2} < {min}", t.id()));
        }
    };
    for (on_what, bg) in [("page", p.background), ("card", p.surface)] {
        for (kind, name) in [
            (ToggleKind::Checkbox, "checkbox"),
            (ToggleKind::Radio, "radio"),
        ] {
            let k = t.choice_toggle(kind, false, idle);
            need(
                &format!("unchecked {name} on {on_what}"),
                ratio(k.border, bg),
                3.0,
            );
        }
        let sw = t.choice_toggle(ToggleKind::Switch, false, idle);
        let edge = ratio(sw.border, bg).max(ratio(sw.track, bg));
        need(&format!("off switch edge on {on_what}"), edge, 3.0);
        let track = over(sw.track, bg);
        need(
            &format!("off switch handle on its track ({on_what})"),
            over(sw.thumb, track).contrast(track),
            3.0,
        );
        let input = t.input(false);
        let field = over(input.background, bg);
        need(
            &format!("empty field border on {on_what}"),
            over(input.border, field).contrast(field),
            3.0,
        );
        let rail = t.progress(None);
        let shows = ratio(rail.track, bg).max(ratio(rail.border, bg));
        need(&format!("progress rail on {on_what}"), shows, 1.15);
    }
    out
}

#[test]
fn resting_controls_are_visible_in_every_theme() {
    let bad: Vec<String> = dodai_core::themes::all()
        .iter()
        .flat_map(failures)
        .collect();
    assert!(
        bad.is_empty(),
        "{} failures:\n{}",
        bad.len(),
        bad.join("\n")
    );
}

#[test]
fn with_contrast_reaches_the_target_and_keeps_good_colours() {
    let night = Color::hex(0x0a0a0c);
    let lifted = Color::hex(0x333333).with_contrast(night, 3.2);
    assert!(lifted.contrast(night) >= 3.2);
    assert!(lifted.contrast(night) < 3.6, "no further than needed");
    let fine = Color::hex(0x777777);
    assert_eq!(fine.with_contrast(night, 3.0), fine);
    let paper = Color::hex(0xf4f4f4);
    assert!(
        Color::hex(0xdddddd)
            .with_contrast(paper, 3.0)
            .contrast(paper)
            >= 3.0
    );
}
