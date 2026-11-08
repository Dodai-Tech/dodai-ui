//! The Dodai showcase in iced: the same pages, sections and controls as the gpui showcase,
//! built from `dodai-showcase-model` with dodai-iced widgets.
//!
//! Keys: `t` / `T` next / previous theme. Env: `DODAI_THEME` (theme id), `DODAI_CAT`
//! (0 = all, 1.. = category), `DODAI_FILTER` (sidebar filter text), `DODAI_SHOT` (write a PPM
//! screenshot and exit).

mod chrome;
mod render;

use std::collections::HashSet;
use std::time::Instant;

use dodai_iced::IcedTheme;
use dodai_showcase_model::{Action, Showcase, Value};
use iced::widget::{container, stack};
use iced::{event, keyboard, Element, Length, Subscription, Task, Theme as IcedBase};

pub struct App {
    pub model: Showcase,
    pub lt: IcedTheme,
    pub t0: Instant,
    pub t: f32,
    pub tree_toggled: HashSet<String>,
    pub seen_revision: u64,
    /// Which text input has the filter: the model keeps its text.
    pub scroll_id: iced::widget::Id,
    /// Frames drawn (for `DODAI_SHOT`).
    pub frames: u32,
}

#[derive(Debug, Clone)]
pub enum Msg {
    Act(Action),
    Set(String, Value),
    TreeToggle(String),
    Tick(Instant),
    Shot(iced::window::Screenshot),
    Noop,
}

pub fn boot() -> App {
    let model = Showcase::from_env();
    let lt = IcedTheme::new(model.theme());
    App {
        seen_revision: model.revision(),
        model,
        lt,
        t0: Instant::now(),
        t: 0.0,
        tree_toggled: HashSet::new(),
        scroll_id: iced::widget::Id::unique(),
        frames: 0,
    }
}

/// Frames to wait before taking the `DODAI_SHOT` screenshot (`DODAI_SHOT_FRAMES` overrides).
fn shot_after() -> u32 {
    std::env::var("DODAI_SHOT_FRAMES")
        .ok()
        .and_then(|n| n.parse().ok())
        .unwrap_or(40)
}

/// Write `shot` as a binary PPM (convert with `sips -s format png`).
pub fn write_ppm(path: &str, size: iced::Size<u32>, rgba: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    let mut out = std::io::BufWriter::new(std::fs::File::create(path)?);
    write!(out, "P6\n{} {}\n255\n", size.width, size.height)?;
    for px in rgba.chunks_exact(4) {
        out.write_all(&px[..3])?;
    }
    out.flush()
}

pub fn update(app: &mut App, msg: Msg) -> Task<Msg> {
    let mut task = Task::none();
    match msg {
        Msg::Act(a) => app.model.update(a),
        Msg::Set(k, v) => app.model.update(Action::Set(k, v)),
        Msg::TreeToggle(k) => toggle(&mut app.tree_toggled, k),
        Msg::Tick(now) => {
            app.t = now.duration_since(app.t0).as_secs_f32();
            app.frames += 1;
            if app.frames == shot_after() && std::env::var_os("DODAI_SHOT").is_some() {
                task = iced::window::latest()
                    .and_then(iced::window::screenshot)
                    .map(Msg::Shot);
            }
        }
        Msg::Shot(shot) => {
            if let Ok(path) = std::env::var("DODAI_SHOT") {
                if let Err(e) = write_ppm(&path, shot.size, &shot.rgba) {
                    eprintln!("could not write {path}: {e}");
                }
            }
            task = iced::exit();
        }
        Msg::Noop => {}
    }
    if app.seen_revision != app.model.revision() {
        app.seen_revision = app.model.revision();
        app.lt.set_theme(app.model.theme());
    }
    task
}

/// Flip membership of `key`.
pub fn toggle(set: &mut HashSet<String>, key: String) {
    if !set.remove(&key) {
        set.insert(key);
    }
}

fn subscription(_app: &App) -> Subscription<Msg> {
    Subscription::batch([
        iced::window::frames().map(Msg::Tick),
        event::listen_with(|event, status, _id| match event {
            iced::Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Character(c),
                ..
            }) if status == event::Status::Ignored => match c.as_str() {
                "t" => Some(Msg::Act(Action::CycleTheme(true))),
                "T" => Some(Msg::Act(Action::CycleTheme(false))),
                _ => None,
            },
            _ => None,
        }),
    ])
}

fn view(app: &App) -> Element<'_, Msg> {
    let page = container(chrome::window(app))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(app.lt.background());
    stack![page, app.lt.scanlines()].into()
}

fn main() -> iced::Result {
    iced::application(boot, update, view)
        .title("Dodai UI · Iced")
        .window_size((1400.0, 900.0))
        .theme(|app: &App| -> IcedBase { app.lt.iced_theme() })
        .subscription(subscription)
        .run()
}

#[cfg(test)]
mod tests {
    use super::*;
    use dodai_showcase_model::Bind;

    #[test]
    fn ppm_has_a_header_and_drops_alpha() {
        let path = std::env::temp_dir().join("dodai-iced-test.ppm");
        let p = path.to_str().unwrap();
        write_ppm(p, iced::Size::new(2, 1), &[1, 2, 3, 255, 4, 5, 6, 0]).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(&bytes[..11], b"P6\n2 1\n255\n");
        assert_eq!(&bytes[11..], &[1, 2, 3, 4, 5, 6]);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn toggling_twice_restores() {
        let mut s = HashSet::new();
        toggle(&mut s, "a".into());
        assert!(s.contains("a"));
        toggle(&mut s, "a".into());
        assert!(s.is_empty());
    }

    #[test]
    fn set_stores_a_value_and_act_changes_the_page() {
        let mut app = boot();
        update(&mut app, Msg::Set("k".into(), Value::Num(2.5)));
        assert_eq!(app.model.values.num(&Bind::new("k", 0.0)), 2.5);
        update(&mut app, Msg::Act(Action::Category(Some(1))));
        assert_eq!(app.model.category(), Some(1));
    }

    #[test]
    fn a_theme_change_reaches_the_iced_theme() {
        let mut app = boot();
        let before = app.lt.theme().id();
        update(&mut app, Msg::Act(Action::CycleTheme(true)));
        assert_ne!(app.lt.theme().id(), before);
        assert_eq!(app.lt.theme().id(), app.model.theme().id());
        // A control colour retints the primary through the same path.
        let primary = app.lt.theme().palette.primary;
        if let Some(&c) = app.model.colours().get(1) {
            update(&mut app, Msg::Act(Action::Colour(c)));
            assert_ne!(app.lt.theme().palette.primary, primary);
        }
    }

    #[test]
    fn ticks_advance_the_clock() {
        let mut app = boot();
        let later = app.t0 + std::time::Duration::from_millis(1500);
        update(&mut app, Msg::Tick(later));
        assert!((app.t - 1.5).abs() < 1e-3);
    }

    #[test]
    fn noop_changes_nothing() {
        let mut app = boot();
        let rev = app.model.revision();
        update(&mut app, Msg::Noop);
        assert_eq!(app.model.revision(), rev);
        assert!(app.tree_toggled.is_empty());
    }
}
