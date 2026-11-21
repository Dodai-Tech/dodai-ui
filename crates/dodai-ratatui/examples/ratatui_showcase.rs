//! `cargo run -p dodai-ratatui --example ratatui_showcase`
//! Keys: `t` next theme · `T` prev · `←/→` tabs · `q` quit.

use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use dodai_core::{ButtonVariant, ControlState, ThemeSwitcher, Tone};
use dodai_ratatui::{
    tui_chart::{
        Axis, BarChart, BarGroup, Candle, CandlestickChart, Gauge, LineChart, Series, Sparkline,
    },
    tui_checkbox::Checkbox as ThemedCheckbox,
    tui_piechart::{PieChart, PieSlice},
    tui_rating::{Rating, RatingState},
    tui_slider::{Slider, SliderState},
    tui_spinner::{BarSpinner, CircleSpinner, FluxSpinner, LinearSpinner},
    Checkbox, Divider, InputField, Progress, Scanlines, SelectList, Tag, ThemeBadge, Themed,
    Toggle, ToggleKind, TuiTheme,
};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    text::{Line, Span},
    widgets::{ListState, Paragraph},
    DefaultTerminal, Frame,
};

struct App {
    sw: ThemeSwitcher,
    tui: TuiTheme,
    tab: usize,
    list: ListState,
    /// Animation frame for the spinners.
    tick: u64,
}

fn main() -> io::Result<()> {
    let sw = dodai_core::switcher();
    let tui = TuiTheme::new(sw.current().clone());
    let mut app = App {
        sw,
        tui,
        tab: 0,
        list: ListState::default().with_selected(Some(0)),
        tick: 0,
    };
    if let Ok(path) = std::env::var("DODAI_DUMP") {
        // DODAI_THEME=n cycles n themes forward before dumping.
        for _ in 0..std::env::var("DODAI_THEME")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0)
        {
            let t = app.sw.cycle_next().clone();
            app.tui.set_theme(t);
        }
        // DODAI_TAB=n picks the page.
        app.tab = std::env::var("DODAI_TAB")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        return dump_html(&mut app, &path);
    }
    let mut term = ratatui::init();
    let res = run(&mut term, &mut app);
    ratatui::restore();
    res
}

fn run(term: &mut DefaultTerminal, app: &mut App) -> io::Result<()> {
    loop {
        term.draw(|f| draw(f, app))?;
        if !event::poll(std::time::Duration::from_millis(100))? {
            app.tick = app.tick.wrapping_add(1);
            continue;
        }
        if let Event::Key(k) = event::read()? {
            if k.kind != KeyEventKind::Press {
                continue;
            }
            match k.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                KeyCode::Char('t') => {
                    let t = app.sw.cycle_next().clone();
                    app.tui.set_theme(t);
                }
                KeyCode::Char('T') => {
                    let t = app.sw.cycle_prev().clone();
                    app.tui.set_theme(t);
                }
                KeyCode::Right => app.tab = (app.tab + 1) % 3,
                KeyCode::Left => app.tab = (app.tab + 2) % 3,
                KeyCode::Down => app.list.select_next(),
                KeyCode::Up => app.list.select_previous(),
                _ => {}
            }
        }
    }
}

fn draw(f: &mut Frame, app: &mut App) {
    let t = &app.tui;
    let area = f.area();
    t.paint_canvas(f.buffer_mut(), area);

    let [head, body, foot] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Min(5),
        Constraint::Length(1),
    ])
    .areas(area);
    f.render_widget(t.tabs(&["Overview", "Widgets", "Charts"], app.tab), head);
    f.render_widget(
        ThemeBadge::new(t),
        head.offset(ratatui::layout::Offset { x: 0, y: 1 }),
    );

    match app.tab {
        1 => draw_widgets(f, t, body, app.tick),
        2 => draw_charts(f, t, body),
        _ => draw_overview(f, t, body, app.list.selected()),
    }

    f.render_widget(
        t.status_bar(t.theme().name(), "q quit · t theme", foot.width),
        foot,
    );
    f.render_widget(Scanlines::new(t), area);
}

fn draw_overview(f: &mut Frame, t: &TuiTheme, body: Rect, selected: Option<usize>) {
    let [left, right] =
        Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]).areas(body);

    // Left: selectable list + toggles + progress.
    let [list_a, ctl_a] = Layout::vertical([Constraint::Length(8), Constraint::Min(5)]).areas(left);
    let blk = t.block("Corporations", true);
    let inner = blk.inner(list_a);
    f.render_widget(blk, list_a);
    f.render_widget(
        SelectList::new(
            t,
            &["Compact", "Comfortable", "Spacious", "Dense", "Custom"],
        )
        .selected(selected),
        inner,
    );

    let blk = t.block("Controls", false);
    let inner = blk.inner(ctl_a);
    f.render_widget(blk, ctl_a);
    let rows = Layout::vertical([Constraint::Length(1); 8]).split(inner);
    f.render_widget(Checkbox::new(t, "Neural link", true), rows[0]);
    f.render_widget(Checkbox::new(t, "Stealth", false), rows[1]);
    f.render_widget(
        Toggle::new(t, ToggleKind::Radio, true, "Cyan").state(ControlState::Focused),
        rows[2],
    );
    f.render_widget(Toggle::new(t, ToggleKind::Radio, false, "Solo"), rows[3]);
    f.render_widget(
        Toggle::new(t, ToggleKind::Switch, true, "Overclock"),
        rows[4],
    );
    f.render_widget(Progress::new(t, 0.62).percent(), rows[5]);
    f.render_widget(
        Progress::new(t, 0.3).tone(Tone::Danger).label("heat"),
        rows[6],
    );
    f.render_widget(Divider::new(t).label("end"), rows[7]);

    // Right: status panel.
    let inner = t.block("Status", false);
    let inner_area = inner.inner(right);
    f.render_widget(inner, right);
    let rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Length(3),
    ])
    .split(inner_area);
    let tags = |solid: bool| {
        let mut v = Vec::new();
        for (l, tone) in [
            ("online", Tone::Success),
            ("warn", Tone::Warning),
            ("error", Tone::Danger),
            ("info", Tone::Info),
        ] {
            v.extend(Tag::new(t, l, tone).solid(solid).line().spans);
            v.push(Span::raw(" "));
        }
        Line::from(v)
    };
    f.render_widget(tags(true), rows[0]);
    f.render_widget(tags(false), rows[1]);
    f.render_widget(Divider::new(t), rows[2]);
    f.render_widget(
        Paragraph::new("Ready. All systems nominal.").style(t.muted()),
        rows[3],
    );
    let btns = Layout::horizontal([
        Constraint::Length(15),
        Constraint::Length(1),
        Constraint::Length(14),
        Constraint::Length(1),
        Constraint::Length(14),
        Constraint::Min(0),
    ])
    .split(rows[5]);
    f.render_widget(
        t.button_block("Continue", ButtonVariant::Primary, ControlState::Idle),
        btns[0],
    );
    f.render_widget(
        t.button_block("Hack", ButtonVariant::OutlineSecondary, ControlState::Idle),
        btns[2],
    );
    f.render_widget(
        t.button_block("Abort", ButtonVariant::Danger, ControlState::Idle),
        btns[4],
    );
    f.render_widget(
        InputField::new(t, "")
            .placeholder("search the net…")
            .focused(true),
        rows[7],
    );
    f.render_widget(
        InputField::new(t, "v.silverhand").cursor(4).focused(true),
        rows[9],
    );
}

/// The third-party widget crates, all styled with one `.themed(t)` call.
fn draw_widgets(f: &mut Frame, t: &TuiTheme, body: Rect, tick: u64) {
    let [left, right] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(body);

    let blk = t.block("tui-checkbox · tui-slider · tui-rating", true);
    let inner = blk.inner(left);
    f.render_widget(blk, left);
    let rows = Layout::vertical([Constraint::Length(1); 9]).split(inner);
    f.render_widget(ThemedCheckbox::new("Neural link", true).themed(t), rows[0]);
    f.render_widget(ThemedCheckbox::new("Stealth", false).themed(t), rows[1]);
    let vol = SliderState::new(65.0, 0.0, 100.0);
    f.render_widget(
        Slider::from_state(&vol)
            .label("Volume")
            .show_value(true)
            .themed(t),
        rows[3],
    );
    let stars = RatingState::new(3.5, 5).with_half_steps(true);
    f.render_widget(
        Rating::from_state(&stars).show_value(true).themed(t),
        rows[5],
    );

    let blk = t.block("tui-spinner", false);
    let inner = blk.inner(right);
    f.render_widget(blk, right);
    let rows = Layout::vertical([Constraint::Length(1); 4]).split(inner);
    f.render_widget(FluxSpinner::new(tick).themed(t), rows[0]);
    f.render_widget(LinearSpinner::new(tick).themed(t), rows[1]);
    f.render_widget(BarSpinner::new(tick).themed(t), rows[2]);
    let ring = Rect::new(
        inner.x,
        inner.y + 4,
        inner.width,
        inner.height.saturating_sub(4),
    );
    f.render_widget(CircleSpinner::new(tick).radius(4).themed(t), ring);
}

/// `tui-chart` and `tui-piechart`.
fn draw_charts(f: &mut Frame, t: &TuiTheme, body: Rect) {
    let [top, bottom] = Layout::vertical([Constraint::Percentage(50); 2]).areas(body);
    let [line_a, bars_a, pie_a] = Layout::horizontal([
        Constraint::Percentage(40),
        Constraint::Percentage(30),
        Constraint::Percentage(30),
    ])
    .areas(top);
    let [candle_a, side_a] =
        Layout::horizontal([Constraint::Percentage(60), Constraint::Percentage(40)]).areas(bottom);

    let wave = |phase: f64| (0..60).map(move |i| (i as f64, (i as f64 / 6.0 + phase).sin()));
    let line = LineChart::new(vec![
        Series::new("alpha").data(wave(0.0)),
        Series::new("beta").data(wave(1.6)),
    ])
    .x_axis(Axis::new().title("t"))
    .y_axis(Axis::new().grid(true))
    .themed(t);
    f.render_widget(&line, t.block("LineChart", false).inner(line_a));
    f.render_widget(t.block("LineChart", false), line_a);

    let bars = BarChart::new(vec![
        BarGroup::new("Q1", [4.0, 7.0]),
        BarGroup::new("Q2", [6.0, 3.0]),
        BarGroup::new("Q3", [9.0, 5.0]),
    ])
    .themed(t);
    f.render_widget(&bars, t.block("BarChart", false).inner(bars_a));
    f.render_widget(t.block("BarChart", false), bars_a);

    let slices = [("Rust", 45.0), ("Go", 25.0), ("C", 20.0), ("Other", 10.0)];
    let pie = PieChart::new(
        slices
            .iter()
            .enumerate()
            .map(|(i, (l, v))| PieSlice::new(l, *v, t.series(i).fg.unwrap_or_default()))
            .collect(),
    )
    .themed(t);
    f.render_widget(&pie, t.block("PieChart", false).inner(pie_a));
    f.render_widget(t.block("PieChart", false), pie_a);

    let candles: Vec<Candle> = (0..24)
        .map(|i| {
            let m = 100.0 + (i as f64 / 3.0).sin() * 8.0 + i as f64 * 0.4;
            let (o, c) = if i % 3 == 0 {
                (m + 2.0, m - 2.0)
            } else {
                (m - 2.0, m + 2.0)
            };
            Candle::new(o, m + 4.0, m - 4.0, c)
        })
        .collect();
    let candle = CandlestickChart::new(candles).themed(t);
    f.render_widget(&candle, t.block("CandlestickChart", false).inner(candle_a));
    f.render_widget(t.block("CandlestickChart", false), candle_a);

    let [spark_a, gauge_a] =
        Layout::vertical([Constraint::Length(4), Constraint::Min(3)]).areas(side_a);
    let spark = Sparkline::new((0..40).map(|i| ((i as f64) / 4.0).sin().abs() * 10.0)).themed(t);
    f.render_widget(&spark, t.block("Sparkline", false).inner(spark_a));
    f.render_widget(t.block("Sparkline", false), spark_a);
    let gauge = Gauge::new(72.0).themed(t);
    f.render_widget(&gauge, t.block("Gauge", false).inner(gauge_a));
    f.render_widget(t.block("Gauge", false), gauge_a);
}

/// Headless render of one frame to HTML (for screenshots / docs): `DODAI_DUMP=out.html`.
fn dump_html(app: &mut App, path: &str) -> io::Result<()> {
    use ratatui::{backend::TestBackend, style::Color, Terminal};
    let (w, h) = (110u16, 36u16);
    let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
    term.draw(|f| draw(f, app)).unwrap();
    let buf = term.backend().buffer().clone();
    let css = |c: Color, d: &str| match c {
        Color::Rgb(r, g, b) => format!("rgb({r},{g},{b})"),
        _ => d.to_string(),
    };
    let mut out = String::from(
        "<body style='margin:0;background:#000'><pre style=\"margin:0;padding:8px;font:16px/1.0 Menlo,monospace;color:#fff\">",
    );
    for y in 0..h {
        for x in 0..w {
            let c = &buf[(x, y)];
            let st = format!(
                "color:{};background:{}",
                css(c.fg, "#fff"),
                css(c.bg, "#000")
            );
            let sym = c.symbol().replace('&', "&amp;").replace('<', "&lt;");
            out.push_str(&format!("<span style=\"{st}\">{sym}</span>"));
        }
        out.push('\n');
    }
    out.push_str("</pre></body>");
    std::fs::write(path, out)
}
