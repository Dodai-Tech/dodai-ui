//! `cargo run -p legendary-ratatui --example ratatui_showcase`
//! Keys: `t` next theme · `T` prev · `←/→` tabs · `q` quit.

use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use legendary_core::{ButtonVariant, ControlState, ThemeSwitcher, Tone};
use legendary_ratatui::{
    Divider, InputField, Progress, Scanlines, SelectList, Tag, ThemeBadge, Toggle, ToggleKind,
    TuiTheme,
};
use ratatui::{
    layout::{Constraint, Layout},
    text::{Line, Span},
    widgets::{Block, ListState, Paragraph},
    DefaultTerminal, Frame,
};

struct App {
    sw: ThemeSwitcher,
    tui: TuiTheme,
    tab: usize,
    list: ListState,
}

fn main() -> io::Result<()> {
    let sw = ThemeSwitcher::new();
    let tui = TuiTheme::new(sw.current().clone());
    let mut app = App {
        sw,
        tui,
        tab: 0,
        list: ListState::default().with_selected(Some(0)),
    };
    if let Ok(path) = std::env::var("LG_DUMP") {
        // LG_THEME=n cycles n themes forward before dumping.
        for _ in 0..std::env::var("LG_THEME")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0)
        {
            let t = app.sw.cycle_next().clone();
            app.tui.set_theme(t);
        }
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
    f.render_widget(Block::new().style(t.base()), area);

    let [head, body, foot] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Min(5),
        Constraint::Length(1),
    ])
    .areas(area);
    f.render_widget(t.tabs(&["Overview", "Network", "Systems"], app.tab), head);
    f.render_widget(
        ThemeBadge::new(t),
        head.offset(ratatui::layout::Offset { x: 0, y: 1 }),
    );

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
            &[
                "Arasaka",
                "Militech",
                "Kang Tao",
                "Biotechnica",
                "Trauma Team",
            ],
        )
        .selected(app.list.selected()),
        inner,
    );

    let blk = t.block("Controls", false);
    let inner = blk.inner(ctl_a);
    f.render_widget(blk, ctl_a);
    let rows = Layout::vertical([Constraint::Length(1); 8]).split(inner);
    f.render_widget(
        Toggle::new(t, ToggleKind::Checkbox, true, "Neural link"),
        rows[0],
    );
    f.render_widget(
        Toggle::new(t, ToggleKind::Checkbox, false, "Stealth"),
        rows[1],
    );
    f.render_widget(
        Toggle::new(t, ToggleKind::Radio, true, "Netrunner").state(ControlState::Focused),
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
            ("breach", Tone::Danger),
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
        Paragraph::new("Wake up, samurai. We have a city to burn.").style(t.muted()),
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
        t.button_block("Jack in", ButtonVariant::Primary, ControlState::Idle),
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

    f.render_widget(
        t.status_bar(t.theme().name(), "q quit · t theme", foot.width),
        foot,
    );
    f.render_widget(Scanlines::new(t), area);
}

/// Headless render of one frame to HTML (for screenshots / docs): `LG_DUMP=out.html`.
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
