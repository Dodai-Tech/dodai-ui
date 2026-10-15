//! `cargo run -p legendary-ratatui --example showcase`
//! Keys: `t` next theme · `T` prev · `←/→` tabs · `q` quit.

use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use legendary_core::{ButtonVariant, ControlState, ThemeSwitcher, Tone};
use legendary_ratatui::{Scanlines, ThemeBadge, TuiTheme};
use ratatui::{
    layout::{Constraint, Layout},
    widgets::{Block, List, ListItem, ListState, Paragraph},
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

    let items: Vec<ListItem> = [
        "Arasaka",
        "Militech",
        "Kang Tao",
        "Biotechnica",
        "Trauma Team",
    ]
    .iter()
    .map(|s| ListItem::new(*s))
    .collect();
    f.render_stateful_widget(
        List::new(items)
            .block(t.block("Corporations", true))
            .highlight_style(t.selected())
            .highlight_symbol("▶ "),
        left,
        &mut app.list,
    );

    let inner = t.block("Status", false);
    let inner_area = inner.inner(right);
    f.render_widget(inner, right);
    let rows = Layout::vertical([Constraint::Length(1); 8]).split(inner_area);
    f.render_widget(
        ratatui::text::Line::from(vec![
            t.badge("online", Tone::Success),
            ratatui::text::Span::raw(" "),
            t.badge("warn", Tone::Warning),
            ratatui::text::Span::raw(" "),
            t.badge("breach", Tone::Danger),
            ratatui::text::Span::raw(" "),
            t.badge("info", Tone::Info),
        ]),
        rows[0],
    );
    f.render_widget(
        Paragraph::new("Wake up, samurai. We have a city to burn.").style(t.muted()),
        rows[2],
    );
    f.render_widget(
        t.button("Jack in", ButtonVariant::Primary, ControlState::Idle),
        rows[4],
    );
    f.render_widget(
        t.button("Hack", ButtonVariant::OutlineSecondary, ControlState::Idle),
        rows[5],
    );
    f.render_widget(
        t.button("Abort", ButtonVariant::Danger, ControlState::Idle),
        rows[6],
    );
    f.render_widget(t.input("", "search the net…", true), rows[7]);

    f.render_widget(
        t.status_bar(t.theme().name(), "q quit · t theme", foot.width),
        foot,
    );
    f.render_widget(Scanlines::new(t), area);
}
