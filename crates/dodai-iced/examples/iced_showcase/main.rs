//! The Dodai showcase in iced: the same pages, sections and controls as the gpui showcase,
//! built from `dodai-showcase-model` with dodai-iced widgets.
//!
//! Keys: `t` / `T` next / previous theme. Env: `DODAI_THEME` (theme id), `DODAI_CAT`
//! (0 = all, 1.. = category), `DODAI_FILTER` (sidebar filter text), `DODAI_SHOT` (write a PPM
//! screenshot and exit).

mod chrome;
mod render;
mod shaders;

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
    /// The page content, measured to place sections.
    pub content_id: iced::widget::Id,
    /// Where the page is scrolled to, in px.
    pub offset: f32,
    /// Frames drawn (for `DODAI_SHOT`).
    pub frames: u32,
    /// Window size, px (frosted surfaces line their backdrop up with it).
    pub window: iced::Size,
    /// The frost demo's picture is behind the app, for this theme id.
    pub wallpaper: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub enum Msg {
    Act(Action),
    Set(String, Value),
    /// Several values at once (the two thumbs of a range).
    SetAll(Vec<(String, Value)>),
    TreeToggle(String),
    Tick(Instant),
    /// The page scrolled: find which section is at the top.
    Scrolled(f32),
    /// Section and content positions, measured for a jump (`true`) or the scroll-spy.
    Measured(Vec<Hit>, bool),
    Shot(iced::window::Screenshot),
    Resized(iced::Size),
    Noop,
}

/// A measured widget of the page.
#[derive(Debug, Clone)]
pub enum Hit {
    /// The scrollable's content box.
    Content(f32),
    /// A section by title and top edge.
    Section(&'static str, f32),
}

pub fn boot() -> App {
    let model = Showcase::from_env();
    let lt = IcedTheme::new(model.theme());
    let mut app = App {
        seen_revision: model.revision(),
        model,
        lt,
        t0: Instant::now(),
        t: 0.0,
        tree_toggled: HashSet::new(),
        scroll_id: iced::widget::Id::unique(),
        content_id: iced::widget::Id::unique(),
        offset: 0.0,
        frames: 0,
        window: iced::Size::new(1400.0, 900.0),
        wallpaper: None,
    };
    sync_glass(&mut app);
    app
}

/// Hand the Surface effects page's mix, the frost demo's picture and the viewport to the theme.
fn sync_glass(app: &mut App) {
    app.lt.set_surface_mix(app.model.surface_fx());
    let theme = app.model.theme();
    let want = app.model.wallpaper().then(|| theme.id());
    if want != app.wallpaper {
        app.wallpaper = want;
        app.lt.set_backdrop(want.map(|_| {
            let p = dodai_showcase_model::media::wallpaper(&theme);
            (p.rgba.to_vec(), p.width as usize, p.height as usize)
        }));
    }
    app.lt
        .set_viewport(app.window, iced::Vector::new(0.0, app.offset));
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
        Msg::Act(a) => {
            app.model.update(a);
            if app.model.take_jump().is_some() {
                task = measure(app, true);
            }
        }
        Msg::Scrolled(y) => {
            app.offset = y;
            task = measure(app, false);
        }
        Msg::Measured(hits, jump) => task = placed(app, &hits, jump),
        Msg::Set(k, v) => app.model.update(Action::Set(k, v)),
        Msg::SetAll(kv) => {
            for (k, v) in kv {
                app.model.update(Action::Set(k, v));
            }
        }
        Msg::TreeToggle(k) => toggle(&mut app.tree_toggled, k),
        Msg::Tick(now) => {
            app.t = now.duration_since(app.t0).as_secs_f32();
            shaders::tick();
            app.frames += 1;
            if app.frames == 8 {
                // `DODAI_SECTION=<title>`: pick that section, as a click in the sidebar would.
                if let Some(t) = std::env::var("DODAI_SECTION").ok().and_then(|t| {
                    app.model
                        .visible()
                        .iter()
                        .find(|s| s.title == t)
                        .map(|s| s.title)
                }) {
                    return update(app, Msg::Act(Action::Section(t)));
                }
            }
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
        Msg::Resized(size) => app.window = size,
        Msg::Noop => {}
    }
    if app.seen_revision != app.model.revision() {
        app.seen_revision = app.model.revision();
        app.lt.set_theme(app.model.theme());
        // The picture is drawn in the theme's colours.
        app.wallpaper = None;
    }
    sync_glass(app);
    task
}

/// Ids of the sections on show, by title.
pub fn section_ids(app: &App) -> Vec<(&'static str, iced::widget::Id)> {
    app.model
        .visible()
        .iter()
        .map(|s| (s.title, iced::widget::Id::from(format!("sec-{}", s.title))))
        .collect()
}

/// Ask the widget tree where the page content and every section are.
fn measure(app: &App, jump: bool) -> Task<Msg> {
    let ids = section_ids(app);
    let content = app.content_id.clone();
    iced::widget::selector::find_all(move |c: iced::widget::selector::Candidate<'_>| {
        let id = c.id()?;
        if *id == content {
            return Some(Hit::Content(c.bounds().y));
        }
        ids.iter()
            .find(|(_, i)| i == id)
            .map(|(t, _)| Hit::Section(t, c.bounds().y))
    })
    .map(move |hits| Msg::Measured(hits, jump))
}

/// Scroll to the picked section, or follow the scroll in the sidebar.
fn placed(app: &mut App, hits: &[Hit], jump: bool) -> Task<Msg> {
    let top = hits.iter().find_map(|h| match h {
        Hit::Content(y) => Some(*y),
        Hit::Section(..) => None,
    });
    let Some(top) = top else {
        return Task::none();
    };
    let mut sections: Vec<(&'static str, f32)> = hits
        .iter()
        .filter_map(|h| match h {
            Hit::Section(t, y) => Some((*t, y - top)),
            Hit::Content(_) => None,
        })
        .collect();
    sections.sort_by(|a, b| a.1.total_cmp(&b.1));
    if jump {
        let wanted = app.model.active_section();
        let at = sections
            .iter()
            .find(|(t, _)| Some(*t) == wanted)
            .map(|s| s.1);
        return at.map_or_else(Task::none, |y| {
            app.offset = y;
            iced::widget::operation::scroll_to(
                app.scroll_id.clone(),
                iced::widget::scrollable::AbsoluteOffset { x: 0.0, y },
            )
        });
    }
    // The last section that has reached the top of the view.
    let current = sections
        .iter()
        .rev()
        .find(|(_, y)| *y <= app.offset + 24.0)
        .or(sections.first())
        .map(|s| s.0);
    if let Some(t) = current {
        if app.model.active_section() != Some(t) {
            app.model.update(Action::Visible(t));
        }
    }
    Task::none()
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
        iced::window::resize_events().map(|(_, size)| Msg::Resized(size)),
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
    // The canvas, the backdrop (the design's mesh or the frost demo's picture), then the app.
    let canvas = container("")
        .width(Length::Fill)
        .height(Length::Fill)
        .style(app.lt.background());
    let mut layers = stack![canvas];
    if let Some(b) = app.lt.backdrop_view() {
        layers = layers.push(b);
    }
    let page = container(chrome::window(app))
        .width(Length::Fill)
        .height(Length::Fill);
    layers.push(page).push(app.lt.scanlines()).into()
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
