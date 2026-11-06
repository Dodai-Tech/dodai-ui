//! # dodai-gpui
//!
//! Turns a [`dodai_core::Theme`] into GPUI values: colours, a [`Global`]
//! theme store (`cx.dodai()`), and a [`DodaiStyled`] extension for
//! any [`Styled`] element (`div().dodai_panel(&theme)`, `.dodai_button(...)`).
//!
//! ## Components
//!
//! Every component is a `RenderOnce` builder reading the active theme from the
//! global ([`install`] first). Shapes (square / rounded / chamfered), glow, hard shadow,
//! shadows, frosted glass and uppercase labels all come from tokens and the active design's
//! spec; no component looks at a theme id.
//!
//! | Area | Modules |
//! |---|---|
//! | Actions | [`button`], [`surfaces`] (icon button, FAB, cards, app bars), [`expressive`] (button group, split button; Material) |
//! | Inputs | [`controls`], [`forms`], [`select`], [`picker`], [`range`], [`knob`], [`text_input`] |
//! | Navigation | [`tabs`], [`nav`], [`navigation`], [`menu`], [`command`], [`search`] |
//! | Data display | [`data`], [`datatable`], [`display`], [`content`], [`list`], [`layout`] (tree, split pane, page grid) |
//! | Feedback | [`feedback`], [`overlay`], [`popover`], [`sheets`] |
//! | Surface effects | [`mod@surface_fx`] (textures, glitch, sheen, flicker, bloom over any filled surface) |
//! | Time | [`time`] (clock, timer, stopwatch, tap tempo) |
//! | Look | [`paint`] (the shaped box), [`motion`], [`frosted`] (glass over a picture), [`glass`] (frosted glass on any component), [`mod@backdrop`] (the canvas mesh of a design that has one) |
//!
//! Charts and instruments are separate crates built on the same pieces: `dodai-gpui-charts` and
//! `dodai-gpui-instruments`. The pieces they share are public here so other crates can build
//! their own: [`paint`] (strokes, fills, shaped boxes), [`plot`] (chart glow), [`follow`] (values
//! that ease to new data), [`motion`], [`live`] (streaming samples) and the [`entry_render!`],
//! [`follow_api!`], [`callback!`] and [`corners_setter!`] macros.
//!
//! ## Design-exclusive components
//!
//! A few components belong to one design and are compiled only with its feature (and the
//! showcase lists them only for that design):
//!
//! | Feature | Components |
//! |---|---|
//! | `material` | [`expressive`]: `ButtonGroup`, `SplitButton`, `LoadingIndicator`; `overscroll` |
//! | `cyberpunk` | `scanlines` |
//! | `dodai` | `page_grid` (`PageGrid`); turns on `frost` (`FrostedScene`) |
//! | `neobrutalism` | none; its look is tokens and spec only |
//!
//! Everything else is shared by all designs and looks different only through tokens.
//!
//! `use dodai_gpui::prelude::*;` brings in the everyday names.
//!
//! ## Conventions
//!
//! * **Constructors**: `Type::new(id, ..)`; the id (`impl Into<ElementId>`) keys the widget's
//!   state. Stateless display types take their content instead.
//! * **Controlled state**: the caller owns the value and passes it back in.
//!   - `selected(..)` is the chosen item (an index, an `Option<usize>`, a set of indices, or a
//!     `bool` on a single item).
//!   - `expanded(..)` is the open nodes, `open(bool)` is a popup.
//!   - `value(..)` is a typed value.
//! * **Callbacks** (`on_*`, always last in the argument list: `(.., &mut Window, &mut App)`):
//!   - `on_click(&ClickEvent)`: pressed.
//!   - `on_select(index or item)`: one item picked out of several.
//!   - `on_change(value)`: a value (number, text, date, set of indices) edited.
//!   - `on_dismiss()`: a surface asked to close.
//! * **Sizes**: `size(ControlSize | FabSize | AppBarSize)` is a named step; `size(side)` is a
//!   square of that many px; `size(w, h)` a rectangle; `width(px)` / `height(px)` one side.
//! * **Surface**: `surface(dodai_core::Surface)` gives one component its own texture and
//!   effects over the app-wide mix (`Surface::NONE` turns them off). Button, IconButton, Card,
//!   Panel, Tag, TextInput, Select, Toolbar, ListItem (lit rows), DialogCard/Dialog and the sheets take it; `corners`, `crop` and `cut_gap` are the
//!   per-component shape overrides.
//! * **Booleans** are `on`/`off` setters (`disabled(true)`, `legend(false)`).
//!
//! ```no_run
//! use gpui::{div, prelude::*, App};
//! use dodai_core::{ButtonVariant, ControlState, Theme};
//! use dodai_gpui::{install, DodaiStyled, ActiveDodai};
//!
//! fn setup(cx: &mut App) {
//!     install(cx, Theme::default());
//!     let t = cx.dodai().theme().clone();
//!     let _btn = div().dodai_button(&t, ButtonVariant::Primary, ControlState::Idle);
//! }
//! ```
#![warn(missing_docs)]
/// Adds the standard `.corners(..)` per-component override (cut these
/// corners instead of the theme's). The type needs a `corners` field of
/// type `Option<dodai_core::Corners>`.
#[macro_export]
macro_rules! corners_setter {
    ($ty:ident) => {
        impl $ty {
            /// Cut these corners instead of the theme's.
            pub fn corners(mut self, c: dodai_core::Corners) -> Self {
                self.corners = Some(c);
                self
            }
        }
    };
}

/// Adds the standard `.surface(..)` per-component override of the texture and effects.
/// The type needs a `surface` field of type `dodai_core::Surface`.
macro_rules! surface_setter {
    ($ty:ident) => {
        impl $ty {
            /// Texture and effects for this component, over the app-wide mix. Anything not
            /// set follows the app, then the design. `Surface::NONE` turns them off.
            pub fn surface(mut self, s: dodai_core::Surface) -> Self {
                self.surface = s;
                self
            }
        }
    };
}
pub(crate) use surface_setter;

/// `crop` / `cut_gap` setters for a control with `crop: Option<Crop>` and `cut_gap: Option<f32>`.
#[cfg(feature = "components")]
macro_rules! crop_setters {
    ($ty:ident) => {
        impl $ty {
            /// Crop an end, drawn the way the current design crops it; see `Button::crop`.
            pub fn crop(mut self, c: $crate::Crop) -> Self {
                self.crop = Some(c);
                self
            }
            /// Gap in px between the bars of the edge tick; see `Button::cut_gap`.
            pub fn cut_gap(mut self, px: f32) -> Self {
                self.cut_gap = Some(px.max(0.0));
                self
            }
        }
    };
}
#[cfg(feature = "components")]
pub(crate) use crop_setters;

/// `impl RenderOnce` that builds via `self.build(window, cx)` and wraps the
/// result in the theme's entry animation, keyed by the type name.
#[macro_export]
macro_rules! entry_render {
    ($ty:ident) => {
        impl gpui::RenderOnce for $ty {
            fn render(
                self,
                window: &mut gpui::Window,
                cx: &mut gpui::App,
            ) -> impl gpui::IntoElement {
                let motion = $crate::motion::current(cx);
                $crate::motion::entry(self.build(window, cx), stringify!($ty), motion)
            }
        }
    };
}

/// Builder setter that stores a callback: `callback!(on_change, usize)`
/// expands to `pub fn on_change(self, f: impl Fn(usize, &mut Window, &mut App))`
/// storing `Some(Rc::new(f))` in the field of the same name.
#[macro_export]
macro_rules! callback {
    ($(#[$m:meta])* $name:ident $(, $arg:ty)*) => {
        $(#[$m])*
        pub fn $name(
            mut self,
            f: impl Fn($($arg,)* &mut gpui::Window, &mut gpui::App) + 'static,
        ) -> Self {
            self.$name = Some(std::rc::Rc::new(f));
            self
        }
    };
}

#[cfg(any(feature = "shaders", feature = "dodai", feature = "frost"))]
pub mod backdrop;
#[cfg(feature = "components")]
pub mod button;
#[cfg(feature = "components")]
pub mod carousel;
#[cfg(feature = "components")]
pub mod command;
#[cfg(feature = "components")]
pub mod content;
#[cfg(feature = "components")]
pub mod controls;
#[cfg(feature = "components")]
pub mod count;
#[cfg(feature = "components")]
pub mod data;
#[cfg(feature = "components")]
pub mod datatable;
#[cfg(feature = "components")]
pub mod display;
#[cfg(feature = "components")]
#[cfg(all(feature = "components", feature = "material"))]
pub mod expressive;
#[cfg(feature = "components")]
pub mod extras;
#[cfg(feature = "components")]
pub mod feedback;
/// The width of a hairline: dividers, rules and thin borders.
pub const HAIRLINE: gpui::Pixels = gpui::px(1.0);

pub mod direction;
pub mod follow;
#[cfg(feature = "components")]
pub mod forms;
#[cfg(feature = "frost")]
pub mod frosted;
#[cfg(feature = "frost")]
pub mod glass;
#[cfg(feature = "components")]
pub mod icons;
pub mod live;
#[cfg(feature = "components")]
mod own;
#[cfg(feature = "components")]
pub mod roving;
pub use live::{Feed, Live};
pub mod interact;
#[cfg(feature = "components")]
pub mod knob;
#[cfg(feature = "components")]
pub mod layout;
pub mod lazy;
#[cfg(feature = "components")]
pub mod list;
pub mod logic;
#[cfg(any(feature = "frost", feature = "components"))]
mod lru;
#[cfg(feature = "components")]
pub mod markdown;
#[cfg(feature = "components")]
pub mod menu;
pub mod motion;
#[cfg(feature = "components")]
pub mod nav;
#[cfg(feature = "components")]
pub mod navigation;
#[cfg(feature = "components")]
pub mod orientation;
#[cfg(feature = "components")]
pub mod overlay;
#[cfg(all(feature = "components", feature = "material"))]
pub mod overscroll;
#[cfg(feature = "dodai")]
pub mod page_grid;
pub mod paint;
pub mod panel;
#[cfg(feature = "components")]
pub mod picker;
pub mod plot;
#[cfg(feature = "components")]
pub mod popover;
#[cfg(feature = "components")]
pub mod range;
#[cfg(feature = "cyberpunk")]
pub mod scanlines;
#[cfg(feature = "components")]
pub mod search;
#[cfg(feature = "components")]
pub mod select;
#[cfg(feature = "components")]
pub mod selection;
#[cfg(feature = "shaders")]
pub mod shader_view;
#[cfg(feature = "components")]
pub mod sheets;
#[cfg(feature = "components")]
pub mod shell;
#[cfg(feature = "components")]
pub mod spy;
pub mod surface_fx;
#[cfg(feature = "components")]
pub mod surfaces;
#[cfg(feature = "components")]
pub mod switcher;
#[cfg(feature = "components")]
pub mod tabs;
#[cfg(feature = "components")]
pub mod tag;
pub mod text;
#[cfg(feature = "components")]
pub mod text_input;
#[cfg(feature = "components")]
pub mod time;

#[cfg(test)]
mod testing;

#[cfg(feature = "components")]
pub use button::{
    button_crop, button_cut_gap, reset_button_crop, set_button_crop, set_button_cut_gap,
    set_design_button_crop, Button, Crop,
};
#[cfg(feature = "components")]
pub use carousel::{layout_widths, window_start, Carousel, CarouselLayout, SMALL_ITEM};
#[cfg(feature = "components")]
pub use command::{fuzzy_filter, Combobox, Command, CommandPalette};
#[cfg(feature = "components")]
pub use content::{
    highlight, parse_blocks, parse_inline, CodeBlock, CodeToken, Markdown, MdBlock, MdInline,
};
#[cfg(feature = "components")]
pub use controls::{
    tick_fractions, BubblePlacement, Checkbox, CheckboxVariant, Orientation, ProgressBar, Radio,
    Slider, Toggle,
};
#[cfg(feature = "components")]
pub use count::CountUp;
#[cfg(feature = "components")]
pub use data::{Avatar, Badge, Chip, Kbd, Table};
#[cfg(feature = "components")]
pub use datatable::{
    compare_cells, max_scroll, next_sort, resized, view_indices, visible_range, Column, DataTable,
};
#[cfg(feature = "components")]
pub use display::{
    arc_points, avatar_split, thumb_geometry, AvatarGroup, Banner, CircularProgress, Image,
    ImageFit, ScrollArea,
};
pub use dodai_core::motion as dodai_motion;
pub use dodai_core::{ClusterKind, DialLook, NeedleResponse};
#[cfg(feature = "shaders")]
pub use dodai_shaders;
#[cfg(feature = "components")]
pub use extras::{
    step_value, BottomAppBar, ColorPicker, EmptyState, FileDropZone, NumberStepper, Rating,
    TagField, Timeline, TimelineEvent, Toolbar,
};
#[cfg(feature = "components")]
pub use feedback::{Alert, IndeterminateBar, Skeleton, Spinner, Toast};
#[cfg(feature = "components")]
pub use forms::{
    otp_push, select_all_state, toggle_select, CheckboxGroup, FieldState, FormField, OtpInput,
    ToggleGroup,
};
#[cfg(all(feature = "components", feature = "cyberpunk"))]
pub use icons::Modifier;
#[cfg(feature = "components")]
pub use icons::{Icon, IconId, IconName, IconPack, SymbolState};
#[cfg(feature = "components")]
mod icon_names;
#[cfg(any(feature = "shaders", feature = "dodai", feature = "frost"))]
pub use backdrop::backdrop;
pub use direction::{direction, is_rtl, set_direction, Direction};
#[cfg(feature = "components")]
#[cfg(all(feature = "components", feature = "material"))]
pub use expressive::{ButtonGroup, LoadingIndicator, SplitButton};
#[cfg(feature = "frost")]
pub use frosted::FrostedScene;
#[cfg(feature = "frost")]
pub use glass::{clear_wallpaper, has_wallpaper, set_wallpaper};
#[cfg(feature = "components")]
pub use knob::Knob;
#[cfg(feature = "components")]
pub use layout::{
    clamp_ratio, item_at, visible_rows, SplitPane, TreeItem, TreeNavigation, TreeRow, TreeView,
};
pub use lazy::lazy;
#[cfg(feature = "components")]
pub use list::{Divider, List, ListItem};
#[cfg(feature = "components")]
pub use menu::{Menu, MenuItem};
pub use motion::{
    entry as entry_animation, grow, lerp, reduce_motion, set_max_fps, set_reduce_motion,
    stagger_window, Curve, Easing, EntryKind, EntryScope, EntrySet, Motion, MotionExt, Tween,
};
#[cfg(feature = "components")]
pub use nav::{Accordion, Breadcrumb, Pagination, Segmented, Stepper};
#[cfg(feature = "components")]
pub use navigation::{NavigationBar, NavigationDrawer, NavigationItem, NavigationRail};
#[cfg(feature = "components")]
pub use orientation::AxisGizmo;
#[cfg(feature = "components")]
pub use overlay::{Dialog, DialogCard};
#[cfg(all(feature = "components", feature = "material"))]
pub use overscroll::{Overscroll, OverscrollFeel};
#[cfg(feature = "dodai")]
pub use page_grid::PageGrid;
pub use paint::Playback;
pub use paint::{shaped, ShapeEdge, ShapeStyle};
pub use panel::{
    section_border, set_section_border, AccentCorner, BorderStyle, DodaiTooltip, InputBox, Panel,
};
#[cfg(feature = "components")]
pub use picker::{
    days_in_month, format_date, format_date_time, format_date_time_range, format_time, month_grid,
    parse_date, parse_time, range_pick, shift_month, step_hour, step_minute, to_12h, weekday, Date,
    DateInput, DatePicker, DateRange, DateTimePicker, DateTimeRangePicker, TimeInput, TimePicker,
};
#[cfg(feature = "components")]
pub use popover::{ContextMenu, DropdownMenu, HoverCard, Popover, PopoverPlacement};
#[cfg(feature = "components")]
pub use range::{nearest_thumb, range_update, RangeSlider};
#[cfg(feature = "cyberpunk")]
pub use scanlines::Scanlines;
#[cfg(feature = "components")]
pub use search::{count_label, highlight_runs, IconBadge, RichTooltip, SearchMode, SearchView};
#[cfg(feature = "components")]
pub use select::{family_title, DesignSystemSelector, Select, SelectOption, ThemeSelector};
#[cfg(feature = "components")]
pub use selection::{SelectionMarker, SelectionTrack};
#[cfg(feature = "shaders")]
pub use shader_view::{shader_backend, PaletteSource, ShaderView};
#[cfg(feature = "components")]
pub use sheets::{
    sheet_style, BottomSheet, FullScreenDialog, ModalDrawer, SheetEdge, SideSheet, Snackbar,
};
#[cfg(feature = "components")]
pub use shell::{is_wide, snap_collapsed, spring_step, ResponsiveShell};
#[cfg(feature = "components")]
pub use spy::{pick as spy_pick, ScrollSpy};
pub use surface_fx::{
    apply_surface_preset, live_press_count, set_surface_fx, surface_fx, texture_swatch,
};
#[cfg(feature = "components")]
pub use surfaces::{
    card_style, AppBarSize, Card, CardVariant, Fab, FabMenu, FabSize, IconButton, IconWidth,
    SearchBar, TopAppBar,
};
#[cfg(feature = "components")]
pub use switcher::ThemeSwitcherBar;
#[cfg(feature = "components")]
pub use tabs::Tabs;
#[cfg(feature = "components")]
pub use tag::Tag;
pub use text::{Heading, Label, Mono, Muted, Text};
#[cfg(feature = "components")]
pub use text_input::TextInput;
#[cfg(feature = "components")]
pub use time::{
    beat_phase, civil_from_days, countdown_text, date_text, day_secs, hand_angles, hms, lap_splits,
    local_utc_offset_minutes, segments, stopwatch_text, tap_bpm, tap_jitter, tempo_name,
    weekday_name, Clock, ClockFace, Stopwatch, TapTempo, Timer,
};

use dodai_core::{ButtonVariant, Color, ControlState, Theme, ThemeSwitcher, Tone};
use gpui::{px, App, BoxShadow, Global, Hsla, Rgba, Styled};

/// Token colour → GPUI [`Rgba`].
pub fn rgba(c: Color) -> Rgba {
    let [r, g, b, a] = c.to_f32();
    Rgba { r, g, b, a }
}

/// Token colour → GPUI [`Hsla`] (what `Styled` methods take).
pub fn hsla(c: Color) -> Hsla {
    rgba(c).into()
}

/// Global theme state: the [`ThemeSwitcher`] plus the active theme.
pub struct DodaiTheme {
    switcher: ThemeSwitcher,
}

impl Global for DodaiTheme {}

impl DodaiTheme {
    /// The active theme.
    pub fn theme(&self) -> &Theme {
        self.switcher.current()
    }

    /// The theme switcher.
    pub fn switcher(&self) -> &ThemeSwitcher {
        &self.switcher
    }

    /// Hsla for a status tone.
    pub fn tone(&self, tone: Tone) -> Hsla {
        hsla(self.theme().palette.tone(tone))
    }
}

/// Install the global theme with `initial` as the only theme. Use
/// [`install_switcher`] to offer several (e.g. `dodai_core::switcher()`).
pub fn install(cx: &mut App, initial: Theme) {
    install_switcher(cx, ThemeSwitcher::with_themes(vec![initial]));
}

/// Install the global theme from a ready [`ThemeSwitcher`] (its current
/// theme is the active one).
///
/// Overrides made with [`set_corners`] belong to the previous themes and are dropped.
pub fn install_switcher(cx: &mut App, switcher: ThemeSwitcher) {
    cx.set_global(DodaiTheme { switcher });
    if cx.has_global::<CornerBase>() {
        cx.remove_global::<CornerBase>();
    }
    // Text-input editing keys (backspace, arrows, enter, clipboard) are
    // actions: without bindings only plain typing would work.
    #[cfg(feature = "components")]
    if !cx.has_global::<KeysBound>() {
        text_input::bind_keys(cx);
        cx.set_global(KeysBound);
    }
}

/// Marker: the text-input key bindings were registered (once per app).
#[cfg(feature = "components")]
struct KeysBound;
#[cfg(feature = "components")]
impl gpui::Global for KeysBound {}

/// Clone of the active theme (components call this on render).
pub fn theme_of(cx: &App) -> Theme {
    let t = cx.dodai().theme();
    let tune = shape_tune();
    let mut t = t.tuned(&tune);
    if chart_monochrome() {
        t.palette = t.palette.monochrome();
    }
    t
}

thread_local! {
    static MONO: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Whether series colours (charts, tags, graph lanes) are drawn as a grey ramp.
pub fn chart_monochrome() -> bool {
    MONO.with(std::cell::Cell::get)
}

/// Draw series colours as a lightness ramp instead of hues, everywhere, and repaint.
pub fn set_chart_monochrome(cx: &mut App, on: bool) {
    MONO.with(|c| c.set(on));
    cx.refresh_windows();
}

thread_local! {
    static TUNE: std::cell::Cell<dodai_core::ShapeTune> =
        const { std::cell::Cell::new(dodai_core::ShapeTune::DEFAULT) };
}

/// The shape tuning applied to every component (roundness and borders).
pub fn shape_tune() -> dodai_core::ShapeTune {
    TUNE.with(std::cell::Cell::get)
}

/// Tune corner roundness and borders for every component and repaint.
pub fn set_shape_tune(cx: &mut App, tune: dodai_core::ShapeTune) {
    TUNE.with(|c| c.set(tune));
    cx.refresh_windows();
}

/// Select theme by index (clamped) and repaint.
pub fn select_theme(cx: &mut App, index: usize) {
    store(cx).switcher.select_index(index);
    cx.refresh_windows();
}

/// Switch to the opposite light/dark mode of the active theme (same design, same accent) and
/// repaint. `false` when the theme has no counterpart.
pub fn toggle_theme_mode(cx: &mut App) -> bool {
    FOLLOWING_SYSTEM.with(|f| f.set(false));
    let done = store(cx).switcher.toggle_mode();
    if done {
        cx.refresh_windows();
    }
    done
}

/// The light/dark mode the OS asks for.
pub fn system_mode(cx: &App) -> dodai_core::Mode {
    match cx.window_appearance() {
        gpui::WindowAppearance::Light | gpui::WindowAppearance::VibrantLight => {
            dodai_core::Mode::Light
        }
        gpui::WindowAppearance::Dark | gpui::WindowAppearance::VibrantDark => {
            dodai_core::Mode::Dark
        }
    }
}

thread_local! {
    /// Whether the theme follows the OS's light/dark mode, and whether the window observer that
    /// does it is installed already.
    static FOLLOWING_SYSTEM: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static SYSTEM_OBSERVER: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Whether the theme follows the OS's light/dark mode ([`follow_system_appearance`]); picking a
/// mode with [`set_theme_mode`] or [`toggle_theme_mode`] ends it.
pub fn following_system_appearance() -> bool {
    FOLLOWING_SYSTEM.with(std::cell::Cell::get)
}

/// Use the active theme's `mode` counterpart (same design, same accent) and stop following the
/// OS. `false` when the theme has no counterpart.
pub fn set_theme_mode(cx: &mut App, mode: dodai_core::Mode) -> bool {
    FOLLOWING_SYSTEM.with(|f| f.set(false));
    let g = store(cx);
    let (before, ok) = (g.switcher.index(), g.switcher.select_mode(mode));
    if g.switcher.index() != before {
        cx.refresh_windows();
    }
    ok
}

/// Switch the active theme to its counterpart in the OS's light/dark mode, now and whenever the
/// OS changes it, until [`set_theme_mode`] or [`toggle_theme_mode`] is used. Call once per window
/// (calling it again only re-applies the OS mode); themes without a counterpart stay as they
/// are. `true` when the active theme matches the OS mode right now.
pub fn follow_system_appearance(window: &mut gpui::Window, cx: &mut App) -> bool {
    fn apply(cx: &mut App) -> bool {
        let mode = system_mode(cx);
        let g = store(cx);
        let (before, ok) = (g.switcher.index(), g.switcher.select_mode(mode));
        if g.switcher.index() != before {
            cx.refresh_windows();
        }
        ok
    }
    FOLLOWING_SYSTEM.with(|f| f.set(true));
    if !SYSTEM_OBSERVER.with(|o| o.replace(true)) {
        window
            .observe_window_appearance(|_, cx| {
                if following_system_appearance() {
                    apply(cx);
                }
            })
            .detach();
    }
    apply(cx)
}

/// Whether the OS asks for reduced motion. Best effort: macOS (Reduce motion), GNOME
/// (animations off) and Windows (animation effects off) are read; elsewhere `false`.
pub fn system_prefers_reduced_motion() -> bool {
    fn out(cmd: &str, args: &[&str]) -> Option<String> {
        let o = std::process::Command::new(cmd).args(args).output().ok()?;
        o.status
            .success()
            .then(|| String::from_utf8_lossy(&o.stdout).trim().to_owned())
    }
    if cfg!(target_os = "macos") {
        out(
            "defaults",
            &["read", "com.apple.universalaccess", "reduceMotion"],
        )
        .is_some_and(|v| v == "1")
    } else if cfg!(target_os = "linux") {
        out(
            "gsettings",
            &["get", "org.gnome.desktop.interface", "enable-animations"],
        )
        .is_some_and(|v| v == "false")
    } else if cfg!(target_os = "windows") {
        out(
            "reg",
            &[
                "query",
                r"HKCU\Control Panel\Desktop\WindowMetrics",
                "/v",
                "MinAnimate",
            ],
        )
        .is_some_and(|v| v.trim_end().ends_with('0'))
    } else {
        false
    }
}

/// Apply the OS's reduced-motion preference (see [`system_prefers_reduced_motion`]) with
/// [`set_reduce_motion`].
pub fn follow_system_motion(cx: &mut App) {
    set_reduce_motion(cx, system_prefers_reduced_motion());
}

/// Current glow intensity of the active theme.
pub fn glow_intensity(cx: &App) -> f32 {
    cx.dodai().theme().effects.glow_intensity
}

/// Set glow intensity on **every** theme (so it survives switching) and repaint.
pub fn set_glow_intensity(cx: &mut App, intensity: f32) {
    map_themes(cx, |_, t| t.with_glow_intensity(intensity));
}

/// Rebuild every installed theme with `f(index, theme)`, keep the active one, and repaint.
fn map_themes(cx: &mut App, f: impl Fn(usize, Theme) -> Theme) {
    let g = store(cx);
    let idx = g.switcher.index();
    let themes = g
        .switcher
        .themes()
        .iter()
        .enumerate()
        .map(|(i, t)| f(i, t.clone()))
        .collect();
    g.switcher = ThemeSwitcher::with_themes(themes);
    g.switcher.select_index(idx);
    cx.refresh_windows();
}

/// Apply a per-component corner override to a theme clone (internal helper).
pub fn apply_corners(mut t: Theme, c: Option<dodai_core::Corners>) -> Theme {
    if let Some(c) = c {
        t.shape.corners = c;
    }
    t
}

/// Corners currently cut by the active theme's chamfered controls.
pub fn corners(cx: &App) -> dodai_core::Corners {
    cx.dodai().theme().shape.corners
}

/// Cut `corners` on **every** theme (so it survives switching) and repaint.
/// Per-component `.corners()` overrides still win.
pub fn set_corners(cx: &mut App, corners: dodai_core::Corners) {
    // Remember each theme's own corners (only those: glow, scanlines, ... set later must
    // survive `reset_corners`).
    if cx.try_global::<CornerBase>().is_none() {
        let base = cx
            .dodai()
            .switcher
            .themes()
            .iter()
            .map(|t| t.shape.corners)
            .collect();
        cx.set_global(CornerBase(base));
    }
    map_themes(cx, |_, t| t.with_corners(corners));
}

/// Each installed theme's own corners, while [`set_corners`] overrides them.
struct CornerBase(Vec<dodai_core::Corners>);
impl gpui::Global for CornerBase {}

/// Whether [`set_corners`] has overridden the design systems' own corners.
pub fn corners_overridden(cx: &App) -> bool {
    cx.try_global::<CornerBase>().is_some()
}

/// Undo [`set_corners`]: every theme gets its own corner shape back.
pub fn reset_corners(cx: &mut App) {
    let Some(base) = cx.try_global::<CornerBase>().map(|b| b.0.clone()) else {
        return;
    };
    cx.remove_global::<CornerBase>();
    map_themes(cx, |i, t| match base.get(i) {
        Some(c) => t.with_corners(*c),
        None => t,
    });
}

/// Whether the fine scanline texture is on for the active theme.
pub fn scanlines_enabled(cx: &App) -> bool {
    cx.dodai().theme().effects.scanlines
}

/// Turn the scanline texture (component bodies + [`Scanlines`] overlay) on/off
/// on **every** theme (so it survives switching) and repaint.
pub fn set_scanlines(cx: &mut App, on: bool) {
    map_themes(cx, |_, t| t.with_scanlines(on));
}

/// Switch to the next theme and repaint.
pub fn cycle_next(cx: &mut App) {
    store(cx).switcher.cycle_next();
    cx.refresh_windows();
}

/// Switch to the previous theme and repaint.
pub fn cycle_prev(cx: &mut App) {
    store(cx).switcher.cycle_prev();
    cx.refresh_windows();
}

/// `cx.dodai()` accessor.
pub trait ActiveDodai {
    /// The Dodai theme store of this context.
    fn dodai(&self) -> &DodaiTheme;
}

impl ActiveDodai for App {
    /// The installed store; without one (no [`install`] / [`install_switcher`] yet), a shared
    /// default over every built-in theme of the enabled designs ([`dodai_core::switcher`]), so
    /// components still paint.
    fn dodai(&self) -> &DodaiTheme {
        static FALLBACK: std::sync::OnceLock<DodaiTheme> = std::sync::OnceLock::new();
        self.try_global::<DodaiTheme>().unwrap_or_else(|| {
            FALLBACK.get_or_init(|| DodaiTheme {
                switcher: dodai_core::switcher(),
            })
        })
    }
}

/// The installed store, installing the default ([`dodai_core::switcher`]) first when there is
/// none, so a theme change works before the app installed its own.
fn store(cx: &mut App) -> &mut DodaiTheme {
    if !cx.has_global::<DodaiTheme>() {
        install_switcher(cx, dodai_core::switcher());
    }
    cx.global_mut::<DodaiTheme>()
}

/// Theme-aware styling for any GPUI [`Styled`] element.
pub trait DodaiStyled: Styled + Sized {
    /// App background + default text colour.
    fn dodai_background(self, t: &Theme) -> Self {
        self.bg(hsla(t.palette.background))
            .text_color(hsla(t.palette.text))
    }

    /// Bordered panel; glow shadow when `focused`.
    fn dodai_panel(self, t: &Theme, focused: bool) -> Self {
        let p = t.panel(focused);
        let s = self
            .bg(hsla(p.background))
            .border_1()
            .border_color(hsla(p.border))
            .text_color(hsla(t.palette.text))
            .rounded(px(t.shape.radius));
        if focused && t.effects.glow_enabled() {
            s.shadow(vec![BoxShadow {
                color: hsla(t.effects.glow_color()),
                offset: gpui::point(px(0.0), px(0.0)),
                blur_radius: px(t.effects.glow_radius()),
                spread_radius: px(0.0),
            }])
        } else {
            s
        }
    }

    /// Button colours + control height for a variant/state.
    fn dodai_button(self, t: &Theme, v: ButtonVariant, state: ControlState) -> Self {
        let b = t.button(v, state);
        let s = self
            .min_h(px(t.shape.control_height))
            .px(px(t.spacing.md))
            .text_color(hsla(b.foreground))
            .rounded(px(t.shape.radius));
        let s = match b.background {
            Some(bg) => s.bg(hsla(bg)),
            None => s.bg(gpui::transparent_black()),
        };
        match b.border {
            Some(c) => s.border_2().border_color(hsla(c)),
            None => s,
        }
    }

    /// Text-input box.
    fn dodai_input(self, t: &Theme, focused: bool) -> Self {
        let i = t.input(focused);
        self.bg(hsla(i.background))
            .border_2()
            .border_color(hsla(i.border))
            .text_color(hsla(i.text))
            .px(px(t.spacing.sm))
            .rounded(px(t.shape.radius))
    }
}

impl<T: Styled + Sized> DodaiStyled for T {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgba_converts() {
        let c = rgba(Color::hex(0xfcee0a));
        assert!((c.r - 0xfc as f32 / 255.0).abs() < 1e-6);
        assert_eq!(c.a, 1.0);
    }

    #[test]
    fn hsla_roundtrip_is_opaque_yellow() {
        let h = hsla(Color::hex(0xfcee0a));
        assert_eq!(h.a, 1.0);
        assert!(h.s > 0.9);
    }

    #[test]
    fn per_component_corners_override_only_when_given() {
        let t = Theme::default();
        let base = t.shape.corners;
        assert_eq!(apply_corners(t.clone(), None).shape.corners, base);
        let all = dodai_core::Corners::ALL;
        assert_eq!(apply_corners(t, Some(all)).shape.corners, all);
    }

    #[test]
    fn glow_intensity_applies_to_all_themes() {
        let mut sw = dodai_core::switcher();
        let themes: Vec<Theme> = sw
            .themes()
            .iter()
            .map(|t| t.clone().with_glow_intensity(2.0))
            .collect();
        sw = ThemeSwitcher::with_themes(themes);
        assert!(sw.themes().iter().all(|t| t.effects.glow_intensity == 2.0));
    }

    #[test]
    fn styled_ext_applies() {
        let t = Theme::default();
        let _ = gpui::div().dodai_panel(&t, true).dodai_button(
            &t,
            ButtonVariant::Outline,
            ControlState::Hovered,
        );
    }
}

/// The everyday names: the theme store and extension traits, and the common components.
pub mod prelude {
    pub use crate::{install, theme_of, ActiveDodai, DodaiStyled};
    #[cfg(feature = "components")]
    pub use crate::{
        Button, Checkbox, Dialog, DialogCard, Divider, Heading, Label, Menu, Muted, Panel,
        ProgressBar, Radio, Segmented, Select, Slider, Tabs, Tag, Toggle,
    };
    pub use dodai_core::{ButtonVariant, ControlSize, ControlState, Theme, Tone};
}

/// The Cyberpunk design (tokens and spec).
#[cfg(feature = "cyberpunk")]
pub use dodai_core::cyberpunk;
/// The Dodai design (tokens and spec), re-exported so the feature alone is enough.
#[cfg(feature = "dodai")]
pub use dodai_core::dodai;
/// The Material design (tokens and spec).
#[cfg(feature = "material")]
pub use dodai_core::material;
/// The Neo-Brutalism design (tokens and spec).
#[cfg(feature = "neobrutalism")]
pub use dodai_core::neobrutalism;

/// Compiles the README's example as a doctest so it cannot rot.
#[cfg(all(doctest, feature = "dodai"))]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;
