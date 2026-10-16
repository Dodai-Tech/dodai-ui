//! Buffer-level tests proving components adapt to theme tokens.

use super::*;
use legendary_core::{themes::Spec, Corners, Mode, Shape, Spacing, ThemeMeta, Typography};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

fn spec(id: &'static str, shape: Shape, upper: bool, scan: bool) -> Theme {
    Spec {
        meta: ThemeMeta {
            id,
            name: id,
            family: "test",
            description: "",
            mode: Mode::Light,
        },
        mode: Mode::Light,
        background: 0xf6f7f9,
        surface: 0xffffff,
        raised: 0xeceef2,
        border: 0xd5d9e0,
        border_strong: 0xaab1bd,
        text: 0x14171c,
        muted: 0x4a5260,
        disabled: 0x8a92a0,
        primary: 0x1f5fd6,
        primary_hover: 0x1a50b5,
        on_primary: 0xffffff,
        secondary: 0x0a7d6e,
        on_secondary: 0xffffff,
        accent: 0xb45309,
        info: 0x1f6fd6,
        success: 0x15803d,
        warning: 0xb45309,
        danger: 0xc81e3a,
        series: [0x1f5fd6; 8],
        typography: Typography {
            display: "x",
            body: "x",
            mono: "x",
            size_xs: 11.0,
            size_sm: 13.0,
            size_md: 15.0,
            size_lg: 18.0,
            size_xl: 24.0,
            size_xxl: 36.0,
            uppercase_labels: upper,
            label_tracking: 0.0,
        },
        spacing: Spacing {
            xs: 4.0,
            sm: 8.0,
            md: 12.0,
            lg: 20.0,
            xl: 32.0,
        },
        shape,
        glow: 0.0,
        glow_blur: 0.0,
        scanlines: scan,
        hard_shadow: 0.0,
    }
    .build()
}

fn shape(corners: Corners, radius: f32, chamfer: f32) -> Shape {
    Shape {
        corners,
        radius,
        chamfer,
        border_width: 1.0,
        control_height: 40.0,
    }
}

fn rounded() -> TuiTheme {
    let t = spec("t-round", shape(Corners::NONE, 8.0, 0.0), false, false);
    TuiTheme::with_depth(t, ColorDepth::TrueColor)
}
fn square() -> TuiTheme {
    let t = spec("t-square", shape(Corners::NONE, 0.0, 0.0), true, true);
    TuiTheme::with_depth(t, ColorDepth::TrueColor)
}
fn cyber() -> TuiTheme {
    TuiTheme::with_depth(Theme::default(), ColorDepth::TrueColor)
}

fn draw(w: impl Widget, width: u16, height: u16) -> Buffer {
    let area = Rect::new(0, 0, width, height);
    let mut b = Buffer::empty(area);
    w.render(area, &mut b);
    b
}

fn row(b: &Buffer, y: u16) -> String {
    (0..b.area.width).map(|x| b[(x, y)].symbol()).collect()
}

#[test]
fn panel_border_follows_shape_kind() {
    for (t, tl, br) in [
        (square(), "┌", "┘"),
        (rounded(), "╭", "╯"),
        (cyber(), "┌", "╱"),
    ] {
        let b = draw(t.block("x", false), 8, 3);
        assert_eq!(b[(0, 0)].symbol(), tl, "{}", t.theme().id());
        assert_eq!(b[(7, 2)].symbol(), br, "{}", t.theme().id());
    }
}

#[test]
fn chamfer_honours_each_cut_corner() {
    let all = border_set(&shape(Corners::ALL, 0.0, 8.0));
    assert_eq!(
        (
            all.top_left,
            all.top_right,
            all.bottom_right,
            all.bottom_left
        ),
        ("╱", "╲", "╱", "╲")
    );
    let diag = border_set(&shape(Corners::DIAGONAL, 0.0, 8.0));
    assert_eq!(
        (
            diag.top_left,
            diag.top_right,
            diag.bottom_right,
            diag.bottom_left
        ),
        ("╱", "┐", "╱", "└")
    );
}

#[test]
fn buttons_cut_only_on_chamfered_themes() {
    let s = |t: &TuiTheme| t.button_block("go", ButtonVariant::Primary, ControlState::Idle);
    assert!(s(&cyber()).lines[2].to_string().ends_with('◤'));
    assert!(!s(&square()).lines[2].to_string().contains('◤'));
    let o = rounded().button_block("go", ButtonVariant::Outline, ControlState::Idle);
    assert_eq!(o.lines[0].to_string().chars().next(), Some('╭'));
    assert_eq!(o.lines[2].to_string().chars().last(), Some('╯'));
    // line button + tabs
    let l = square().button("a", ButtonVariant::Primary, ControlState::Idle);
    assert!(!l.to_string().contains('◤'));
    assert!(!square().tabs(&["a"], 0).to_string().contains('◤'));
    assert!(cyber().tabs(&["a"], 0).to_string().contains('◤'));
}

#[test]
fn uppercase_labels_follow_token() {
    assert_eq!(square().label("hi"), "HI");
    assert_eq!(rounded().label("hi"), "hi");
    let b = draw(Tag::new(&square(), "ok", Tone::Info), 6, 1);
    assert_eq!(row(&b, 0).trim(), "OK");
}

#[test]
fn tag_solid_and_outline() {
    let t = rounded();
    let c = t.theme().palette.tone(Tone::Danger);
    let solid = draw(Tag::new(&t, "err", Tone::Danger), 8, 1);
    assert_eq!(solid[(1, 0)].bg, t.color(c));
    let out = draw(Tag::new(&t, "err", Tone::Danger).outline(), 8, 1);
    assert_eq!(row(&out, 0).trim(), "( err )");
    assert_eq!(out[(2, 0)].fg, t.color(c));
    let ch = Tag::new(&cyber(), "err", Tone::Danger).outline().line();
    assert!(ch.to_string().ends_with('◤'));
}

#[test]
fn toggle_glyphs_and_colours() {
    let t = rounded();
    let on = draw(Toggle::new(&t, ToggleKind::Checkbox, true, "a"), 8, 1);
    let off = draw(Toggle::new(&t, ToggleKind::Checkbox, false, "a"), 8, 1);
    assert!(row(&on, 0).starts_with("[x] a"));
    assert!(row(&off, 0).starts_with("[ ] a"));
    let tk = t.theme().toggle(true, ControlState::Idle);
    assert_eq!(on[(1, 0)].bg, t.color(tk.track));
    assert_eq!(on[(1, 0)].fg, t.color(tk.thumb));
    let r = draw(Toggle::new(&t, ToggleKind::Radio, true, "r"), 8, 1);
    assert!(row(&r, 0).starts_with("(●) r"));
}

#[test]
fn progress_uses_eighth_blocks() {
    let t = square();
    let p = Progress::new(&t, 0.5);
    assert_eq!(p.bar_text(10), "█████     ");
    assert_eq!(Progress::new(&t, 0.125).bar_text(4), "▌   ");
    assert_eq!(Progress::new(&t, 2.0).bar_text(3), "███");
    let b = draw(Progress::new(&t, 0.5).percent(), 14, 1);
    assert!(row(&b, 0).ends_with(" 50%"));
    let pt = t.theme().progress(None);
    assert_eq!(b[(0, 0)].fg, t.color(pt.fill));
    assert_eq!(b[(0, 0)].bg, t.color(pt.track));
}

#[test]
fn select_list_marks_selection() {
    let t = cyber();
    let b = draw(SelectList::new(&t, &["a", "b"]).selected(Some(1)), 6, 2);
    assert_eq!(b[(0, 1)].symbol(), "▌");
    assert_eq!(b[(0, 0)].symbol(), " ");
    let lt = t.theme().list_item(true, false);
    assert_eq!(b[(0, 1)].fg, t.color(lt.marker.unwrap()));
    assert_eq!(b[(3, 1)].bg, t.color(lt.background.unwrap()));
}

#[test]
fn divider_fills_width() {
    let t = rounded();
    let b = draw(Divider::new(&t), 5, 1);
    assert_eq!(row(&b, 0), "─────");
    let b = draw(Divider::new(&t).label("ab"), 12, 1);
    assert!(row(&b, 0).contains(" ab "));
    assert_eq!(row(&b, 0).chars().count(), 12);
}

#[test]
fn input_cursor_and_border() {
    let t = rounded();
    let b = draw(InputField::new(&t, "abc").focused(true).cursor(1), 10, 3);
    assert_eq!(b[(0, 0)].symbol(), "╭");
    assert_eq!(b[(4, 1)].symbol(), "b");
    assert_eq!(b[(4, 1)].bg, t.cursor().bg.unwrap());
    let one = draw(InputField::new(&t, "").placeholder("ph"), 10, 1);
    assert!(row(&one, 0).contains("ph"));
}

#[test]
fn scanlines_only_when_token_set() {
    assert!(square().scanlines_enabled());
    assert!(!rounded().scanlines_enabled());
    let t = rounded();
    let area = Rect::new(0, 0, 1, 2);
    let mut b = Buffer::empty(area);
    b[(0, 1)].set_fg(Color::Rgb(100, 100, 100));
    Scanlines::new(&t).render(area, &mut b);
    assert_eq!(b[(0, 1)].fg, Color::Rgb(100, 100, 100));
    assert_eq!(
        dim_color(Color::Rgb(100, 100, 100), 0.5),
        Color::Rgb(50, 50, 50)
    );
}
