//! Buffer-level tests proving components adapt to theme tokens.

use super::*;
use dodai_core::{themes::ThemeBuilder, Corners, Mode, Shape, ThemeMeta, Typography};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

fn spec(id: &'static str, shape: Shape, upper: bool, scan: bool) -> Theme {
    ThemeBuilder {
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
        shape,
        scanlines: scan,
        hard_shadow: 0.0,
        ..ThemeBuilder::neutral(ThemeMeta {
            id,
            name: id,
            family: "test",
            description: "",
            mode: Mode::Light,
        })
    }
    .build()
}

fn shape(corners: Corners, radius: f32, chamfer: f32) -> Shape {
    Shape {
        corners,
        radius,
        chamfer,
        scoop: false,
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
    TuiTheme::with_depth(
        dodai_core::themes::cyberpunk::yellow(),
        ColorDepth::TrueColor,
    )
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
    let on = draw(Toggle::new(&t, ToggleKind::Radio, true, "a"), 8, 1);
    let off = draw(Toggle::new(&t, ToggleKind::Radio, false, "a"), 8, 1);
    assert!(row(&on, 0).starts_with("(●) a"));
    assert!(row(&off, 0).starts_with("( ) a"));
    let tk = t.theme().toggle(true, ControlState::Idle);
    assert_eq!(on[(1, 0)].bg, t.color(tk.track));
    assert_eq!(on[(1, 0)].fg, t.color(tk.thumb));
    let s = draw(Toggle::new(&t, ToggleKind::Switch, true, "s"), 8, 1);
    assert!(row(&s, 0).starts_with("[ ●] s"));
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
    let lt = t.theme().list_item(dodai_core::ItemState::SELECTED);
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

fn real(t: dodai_core::Theme) -> TuiTheme {
    TuiTheme::with_depth(t, ColorDepth::TrueColor)
}

#[test]
fn checkbox_design_follows_the_look_not_the_theme_id() {
    use dodai_core::themes::{cyberpunk, dodai, material, neobrutalism};
    for (t, want) in [
        (cyberpunk::yellow(), CheckboxDesign::Cyberpunk),
        (material::dark(), CheckboxDesign::Material),
        (material::light(), CheckboxDesign::Material),
        (dodai::dark(), CheckboxDesign::Dodai),
        (neobrutalism::light(), CheckboxDesign::Neobrutalism),
        (neobrutalism::dark(), CheckboxDesign::Neobrutalism),
    ] {
        let id = t.id();
        let tui = real(t);
        assert_eq!(
            Checkbox::new(&tui, "a", true).resolved_design(),
            want,
            "{id}"
        );
    }
    // A theme without a spec resolves to the soft Dodai look.
    assert_eq!(
        Checkbox::new(&rounded(), "a", true).resolved_design(),
        CheckboxDesign::Dodai
    );
}

#[test]
fn checkbox_takes_colours_from_the_palette_tokens() {
    for t in dodai_core::themes::all() {
        let tui = real(t.clone());
        let p = &t.palette;
        let b = draw(Checkbox::new(&tui, "go", true).shadow(false), 12, 1);
        // The mark cell sits on a primary-to-secondary wash in Dodai, flat primary elsewhere.
        let bg = b[(1, 0)].bg;
        let design = Checkbox::new(&tui, "go", true).resolved_design();
        if design != CheckboxDesign::Dodai {
            assert_eq!(bg, tui.color(p.primary), "{}", t.id());
            assert_eq!(b[(1, 0)].fg, tui.color(p.on_primary), "{}", t.id());
        }
        assert!(
            row(&b, 0).contains("go") || row(&b, 0).contains("GO"),
            "{}",
            t.id()
        );
    }
}

#[test]
fn checkbox_states_and_sizes() {
    let tui = real(dodai_core::themes::material::dark());
    let off = draw(Checkbox::new(&tui, "a", false), 12, 1);
    assert!(row(&off, 0).starts_with("▐ ▌"));
    let mixed = draw(Checkbox::new(&tui, "a", false).mixed(true), 12, 1);
    assert_eq!(mixed[(1, 0)].symbol(), "−");
    let blk = Checkbox::new(&tui, "a", false).block(true);
    assert_eq!((blk.width(), blk.height()), (5 + 1 + 1 + 2, 3));
    let dis = draw(
        Checkbox::new(&tui, "a", true).state(ControlState::Disabled),
        12,
        1,
    );
    assert_ne!(dis[(1, 0)].bg, tui.color(tui.theme().palette.primary));
    let focus = draw(
        Checkbox::new(&tui, "a", false).state(ControlState::Focused),
        12,
        1,
    );
    assert_eq!(focus[(0, 0)].fg, tui.color(tui.theme().palette.primary));
}

#[test]
fn checkbox_shadow_follows_the_theme_unless_overridden() {
    let neo = real(dodai_core::themes::neobrutalism::light());
    assert!(neo.theme().effects.hard_shadow > 0.0);
    assert_eq!(Checkbox::new(&neo, "a", true).height(), 2);
    assert_eq!(Checkbox::new(&neo, "a", true).shadow(false).height(), 1);
    let mat = real(dodai_core::themes::material::dark());
    assert_eq!(Checkbox::new(&mat, "a", true).height(), 1);
    assert_eq!(Checkbox::new(&mat, "a", true).shadow(true).height(), 2);
}

#[test]
fn checkbox_label_honours_uppercase_labels_once() {
    let b = draw(Checkbox::new(&square(), "hi", true), 12, 2);
    assert!(row(&b, 0).contains("HI"));
    let l = draw(Checkbox::new(&rounded(), "Hi", true), 12, 1);
    assert!(row(&l, 0).contains("Hi"));
}

#[test]
fn checkbox_line_matches_the_widget_row() {
    let tui = real(dodai_core::themes::cyberpunk::yellow());
    let cb = Checkbox::new(&tui, "arm", true).shadow(false);
    let line = cb.line();
    let b = draw(
        Checkbox::new(&tui, "arm", true).shadow(false),
        cb.width(),
        1,
    );
    assert_eq!(line.to_string(), row(&b, 0));
}
