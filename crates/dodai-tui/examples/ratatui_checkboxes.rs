//! The design-aware `Checkbox` in the four design systems: Cyberpunk, Material, Dodai and
//! Neo-brutalism, each from its own theme tokens.
//!
//! Keys: `s` inline/block, `r` readout, `q` quits.
use dodai_core::{themes, ControlState};
use dodai_ratatui::{Checkbox, TuiTheme};
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Margin, Rect},
    widgets::Widget,
};

/// `(label, checked, mixed, state)` rows of every panel.
const ROWS: [(&str, bool, bool, ControlState); 5] = [
    ("Notifications", true, false, ControlState::Idle),
    ("Auto update", false, false, ControlState::Idle),
    ("Share data", false, true, ControlState::Idle),
    ("Focused", true, false, ControlState::Focused),
    ("Disabled", true, false, ControlState::Disabled),
];

fn main() -> std::io::Result<()> {
    let themes = [
        themes::cyberpunk::yellow(),
        themes::material::dark(),
        themes::dodai::dark(),
        themes::neobrutalism::dark(),
    ]
    .map(TuiTheme::new);
    let (mut block, mut readout) = (false, false);
    let mut terminal = ratatui::init();
    let result = loop {
        let drawn = terminal.draw(|f| {
            let cells = Layout::vertical([Constraint::Fill(1); 2])
                .split(f.area())
                .iter()
                .flat_map(|r| {
                    Layout::horizontal([Constraint::Fill(1); 2])
                        .split(*r)
                        .to_vec()
                })
                .collect::<Vec<_>>();
            for (tui, area) in themes.iter().zip(cells) {
                panel(f.buffer_mut(), area, tui, block, readout);
            }
        });
        if let Err(e) = drawn {
            break Err(e);
        }
        match event::read() {
            Ok(Event::Key(k)) if k.kind == KeyEventKind::Press => match k.code {
                KeyCode::Char('q') | KeyCode::Esc => break Ok(()),
                KeyCode::Char('s') => block = !block,
                KeyCode::Char('r') => readout = !readout,
                _ => {}
            },
            Err(e) => break Err(e),
            _ => {}
        }
    };
    ratatui::restore();
    result
}

fn panel(
    buf: &mut ratatui::buffer::Buffer,
    area: Rect,
    tui: &TuiTheme,
    block: bool,
    readout: bool,
) {
    let frame = tui
        .block(tui.theme().name().to_string(), false)
        .style(tui.base());
    let inner = frame.inner(area).inner(Margin::new(2, 1));
    frame.render(area, buf);
    let mut y = inner.y;
    for (label, checked, mixed, state) in ROWS {
        let cb = Checkbox::new(tui, label, checked)
            .mixed(mixed)
            .state(state)
            .block(block)
            .readout(readout);
        let h = cb.height();
        if y + h > inner.bottom() {
            break;
        }
        cb.render(Rect::new(inner.x, y, inner.width, h), buf);
        y += h + 1;
    }
}
