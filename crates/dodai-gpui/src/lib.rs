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
//! | Charts | [`charts`] and its `charts_*` / `instrument_charts` / [`gauges`] siblings |
//! | Time | [`time`] (clock, timer, stopwatch, tap tempo) |
//! | Look | [`paint`] (the shaped box), [`motion`], [`frosted`] (glass over a picture) |
//!
//! ## Design-exclusive components
//!
//! A few components belong to one design and are compiled only with its feature (and the
//! showcase lists them only for that design):
//!
//! | Feature | Components |
//! |---|---|
//! | `material` | [`expressive`]: `ButtonGroup`, `SplitButton`, `LoadingIndicator`; `overscroll` |
//! | `cyberpunk` | `instrument_charts` (oscilloscope, spectrum, radar scope, hex map, ring stack), `scanlines` |
//! | `aurora` | `frosted` (`FrostedScene`), `page_grid` (`PageGrid`) |
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
pub(crate) use corners_setter;

/// `crop` / `cut_gap` setters for a control with `crop: Option<Crop>` and `cut_gap: Option<f32>`.
macro_rules! crop_setters {
    ($ty:ident) => {
        impl $ty {
            /// Crop an end: cut corner plus edge tick (left/right) or a shallow slot (top/bottom); see `Button::crop`.
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
pub(crate) use crop_setters;

/// `impl RenderOnce` that builds via `self.build(window, cx)` and wraps the
/// result in the theme's entry animation, keyed by the type name.
#[allow(unused_macros)]
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
#[allow(unused_imports)]
pub(crate) use entry_render;

/// Builder setter that stores a callback: `callback!(on_change, usize)`
/// expands to `pub fn on_change(self, f: impl Fn(usize, &mut Window, &mut App))`
/// storing `Some(Rc::new(f))` in the field of the same name.
#[allow(unused_macros)]
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
#[allow(unused_imports)]
pub(crate) use callback;

#[cfg(feature = "components")]
pub mod button;
#[cfg(feature = "components")]
pub mod carousel;
#[cfg(feature = "charts")]
pub mod charts;
#[cfg(feature = "charts")]
pub mod charts_distribution;
#[cfg(feature = "charts")]
pub mod charts_flow;
#[cfg(feature = "charts")]
pub mod charts_market;
#[cfg(feature = "charts")]
pub mod charts_part_whole;
#[cfg(feature = "charts")]
pub mod charts_trend;
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
#[cfg(feature = "components")]
pub mod forms;
#[cfg(feature = "aurora")]
pub mod frosted;
#[cfg(feature = "charts")]
pub mod gauges;
pub mod icons;
#[cfg(all(feature = "charts", feature = "cyberpunk"))]
pub mod instrument_charts;
pub mod interact;
#[cfg(feature = "components")]
pub mod knob;
#[cfg(feature = "components")]
pub mod layout;
pub mod lazy;
#[cfg(feature = "components")]
pub mod list;
pub mod logic;
#[cfg(feature = "components")]
pub mod menu;
pub mod motion;
#[cfg(feature = "components")]
pub mod nav;
#[cfg(feature = "components")]
pub mod navigation;
#[cfg(feature = "components")]
pub mod overlay;
#[cfg(all(feature = "components", feature = "material"))]
pub mod overscroll;
#[cfg(feature = "aurora")]
pub mod page_grid;
pub mod paint;
pub mod panel;
#[cfg(feature = "components")]
pub mod picker;
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
#[cfg(feature = "shaders")]
pub mod shader_surface;
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
pub use button::{button_crop, button_cut_gap, set_button_crop, set_button_cut_gap, Button, Crop};
#[cfg(feature = "components")]
pub use carousel::{layout_widths, window_start, Carousel, CarouselLayout, SMALL_ITEM};
#[cfg(feature = "charts")]
pub use charts::{
    bar_max, fmt_num, heat_color, heat_color_with, marker_points, nice_ticks, norm, pie_fractions,
    pie_hit, polar, smooth_points, spark_columns, BarChart, Gauge, Heatmap, Legend, LineChart,
    Marker, PieChart, PieStyle, RadarChart, ScatterPlot, Series, SparkStyle, Sparkline,
};
#[cfg(feature = "charts")]
pub use charts_distribution::{histogram_bins, quartiles, BoxPlot, BubbleChart, Histogram};
#[cfg(feature = "charts")]
pub use charts_flow::{waterfall_steps, Funnel, Waterfall};
#[cfg(feature = "charts")]
pub use charts_market::{ChartStack, DepthChart, Gantt, RenkoChart};
#[cfg(feature = "charts")]
pub use charts_part_whole::{treemap_layout, RadialBar, Treemap};
#[cfg(feature = "charts")]
pub use charts_trend::{
    overlay_lines, stack_cumulative, Candle, CandleOverlay, CandlePane, CandleStyle, Candlestick,
    StackedAreaChart, TradeMarker,
};
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
#[cfg(feature = "charts")]
pub use dodai_charts_core::gantt::Task as GanttTask;
#[cfg(feature = "charts")]
pub use dodai_charts_core::Gradient;
pub use dodai_motion;
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
#[cfg(feature = "charts")]
pub use gauges::GaugeStyle;
#[cfg(feature = "cyberpunk")]
pub use icons::Modifier;
pub use icons::{Icon, IconId, IconName, IconPack};
#[cfg(all(feature = "charts", feature = "cyberpunk"))]
pub use instrument_charts::{
    Blip, DotMatrix, HexMap, Oscilloscope, RadarScope, RingStack, SignalBars, Spectrum,
};
mod icon_names;
#[cfg(feature = "components")]
#[cfg(all(feature = "components", feature = "material"))]
pub use expressive::{ButtonGroup, LoadingIndicator, SplitButton};
#[cfg(feature = "shaders")]
#[cfg(feature = "aurora")]
pub use frosted::FrostedScene;
#[cfg(feature = "components")]
pub use knob::Knob;
#[cfg(feature = "components")]
pub use layout::{clamp_ratio, visible_rows, SplitPane, TreeItem, TreeRow, TreeView};
pub use lazy::lazy;
#[cfg(feature = "components")]
pub use list::{Divider, ListItem};
#[cfg(feature = "components")]
pub use menu::{Menu, MenuItem};
pub use motion::{
    entry as entry_animation, grow, lerp, stagger_window, Curve, Easing, EntryKind, EntryScope,
    EntrySet, Motion, MotionExt, Tween,
};
#[cfg(feature = "components")]
pub use nav::{Accordion, Breadcrumb, Pagination, Segmented, Stepper};
#[cfg(feature = "components")]
pub use navigation::{NavigationBar, NavigationDrawer, NavigationItem, NavigationRail};
#[cfg(feature = "components")]
pub use overlay::{Dialog, DialogCard};
#[cfg(all(feature = "components", feature = "material"))]
pub use overscroll::{Overscroll, OverscrollFeel};
#[cfg(feature = "aurora")]
pub use page_grid::PageGrid;
pub use paint::Playback;
pub use paint::{shaped, ShapeEdge, ShapeStyle};
pub use panel::{
    section_border, set_section_border, AccentCorner, BorderStyle, DodaiTooltip, InputBox, Panel,
};
#[cfg(feature = "components")]
pub use picker::{
    days_in_month, format_date, format_time, month_grid, parse_date, parse_time, range_pick,
    shift_month, step_hour, step_minute, to_12h, weekday, Date, DateInput, DatePicker, DateRange,
    TimeInput, TimePicker,
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
#[cfg(feature = "shaders")]
pub use shader_surface::ShaderSurface;
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
pub use surface_fx::{apply_surface_preset, set_surface_fx, surface_fx};
#[cfg(feature = "components")]
pub use surfaces::{
    card_style, AppBarSize, Card, CardVariant, Fab, FabMenu, FabSize, IconButton, IconWidth,
    SearchBar, TopAppBar,
};
#[cfg(feature = "components")]
pub use switcher::{set_tree_expanded, tree_expanded, DesignSystemTree, ThemeSwitcherBar};
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
/// [`install_switcher`] to offer several (e.g. `dodai::switcher()`).
pub fn install(cx: &mut App, initial: Theme) {
    install_switcher(cx, ThemeSwitcher::with_themes(vec![initial]));
}

/// Install the global theme from a ready [`ThemeSwitcher`] (its current
/// theme is the active one).
pub fn install_switcher(cx: &mut App, switcher: ThemeSwitcher) {
    cx.set_global(DodaiTheme { switcher });
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
    cx.dodai().theme().clone()
}

/// Select theme by index (clamped) and repaint.
pub fn select_theme(cx: &mut App, index: usize) {
    cx.global_mut::<DodaiTheme>().switcher.select_index(index);
    cx.refresh_windows();
}

/// Current glow intensity of the active theme.
pub fn glow_intensity(cx: &App) -> f32 {
    cx.dodai().theme().effects.glow_intensity
}

/// Set glow intensity on **every** theme (so it survives switching) and repaint.
pub fn set_glow_intensity(cx: &mut App, intensity: f32) {
    let g = cx.global_mut::<DodaiTheme>();
    let idx = g.switcher.index();
    let themes = g
        .switcher
        .themes()
        .iter()
        .map(|t| t.clone().with_glow_intensity(intensity))
        .collect();
    g.switcher = ThemeSwitcher::with_themes(themes);
    g.switcher.select_index(idx);
    cx.refresh_windows();
}

/// Apply a per-component corner override to a theme clone (internal helper).
pub(crate) fn apply_corners(mut t: Theme, c: Option<dodai_core::Corners>) -> Theme {
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
    // Remember the themes as installed, so `reset_corners` can restore them.
    if cx.try_global::<CornerBase>().is_none() {
        let base = cx.global::<DodaiTheme>().switcher.themes().to_vec();
        cx.set_global(CornerBase(base));
    }
    let g = cx.global_mut::<DodaiTheme>();
    let idx = g.switcher.index();
    let themes = g
        .switcher
        .themes()
        .iter()
        .map(|t| t.clone().with_corners(corners))
        .collect();
    g.switcher = ThemeSwitcher::with_themes(themes);
    g.switcher.select_index(idx);
    cx.refresh_windows();
}

struct CornerBase(Vec<Theme>);
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
    let g = cx.global_mut::<DodaiTheme>();
    let idx = g.switcher.index();
    g.switcher = ThemeSwitcher::with_themes(base);
    g.switcher.select_index(idx);
    cx.remove_global::<CornerBase>();
    cx.refresh_windows();
}

/// Whether the fine scanline texture is on for the active theme.
pub fn scanlines_enabled(cx: &App) -> bool {
    cx.dodai().theme().effects.scanlines
}

/// Turn the scanline texture (component bodies + [`Scanlines`] overlay) on/off
/// on **every** theme (so it survives switching) and repaint.
pub fn set_scanlines(cx: &mut App, on: bool) {
    let g = cx.global_mut::<DodaiTheme>();
    let idx = g.switcher.index();
    let themes = g
        .switcher
        .themes()
        .iter()
        .map(|t| t.clone().with_scanlines(on))
        .collect();
    g.switcher = ThemeSwitcher::with_themes(themes);
    g.switcher.select_index(idx);
    cx.refresh_windows();
}

/// Switch to the next theme; call `cx.refresh_windows()` afterwards to repaint.
pub fn cycle_next(cx: &mut App) {
    cx.global_mut::<DodaiTheme>().switcher.cycle_next();
    cx.refresh_windows();
}

/// Switch to the previous theme and repaint.
pub fn cycle_prev(cx: &mut App) {
    cx.global_mut::<DodaiTheme>().switcher.cycle_prev();
    cx.refresh_windows();
}

/// `cx.dodai()` accessor.
pub trait ActiveDodai {
    /// The Dodai theme store of this context.
    fn dodai(&self) -> &DodaiTheme;
}

impl ActiveDodai for App {
    fn dodai(&self) -> &DodaiTheme {
        self.global::<DodaiTheme>()
    }
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
            .h(px(t.shape.control_height))
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
        let mut sw = dodai::switcher();
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
