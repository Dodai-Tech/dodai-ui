//! # dodai-gpui
//!
//! Turns a [`dodai_core::Theme`] into GPUI values: colours, a [`Global`]
//! theme store (`cx.dodai()`), and a [`DodaiStyled`] extension for
//! any [`Styled`] element (`div().dodai_panel(&theme)`, `.dodai_button(...)`).
//!
//! ## Components
//!
//! Every component is a `RenderOnce` builder reading the active theme from the
//! global ([`install`] first): [`Button`], [`Panel`], [`Tag`], [`Tabs`],
//! [`Toggle`], [`Checkbox`], [`Radio`], [`ProgressBar`], [`Slider`], [`InputBox`],
//! [`ListItem`], [`Divider`], [`DodaiTooltip`], [`Heading`]/[`Label`]/[`Muted`]/[`Mono`],
//! [`Scanlines`], [`ThemeSwitcherBar`]. Shapes (square / rounded / chamfered),
//! glow, hard shadow and uppercase labels all come from tokens; no component
//! looks at a theme id. See [`paint`] for the shaped-box primitive.
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
pub mod controls;
#[cfg(feature = "components")]
pub mod count;
#[cfg(feature = "components")]
pub mod data;
#[cfg(feature = "components")]
pub mod extras;
#[cfg(feature = "components")]
pub mod feedback;
pub mod icons;
pub mod interact;
#[cfg(feature = "components")]
pub mod knob;
#[cfg(feature = "components")]
pub mod layout;
#[cfg(feature = "components")]
pub mod list;
pub mod logic;
#[cfg(feature = "components")]
pub mod menu;
pub mod motion;
#[cfg(feature = "components")]
pub mod nav;
#[cfg(feature = "components")]
pub mod overlay;
pub mod paint;
pub mod panel;
#[cfg(feature = "components")]
pub mod picker;
#[cfg(feature = "components")]
pub mod range;
pub mod scanlines;
#[cfg(feature = "components")]
pub mod search;
#[cfg(feature = "components")]
pub mod select;
#[cfg(feature = "shaders")]
pub mod shader_view;
#[cfg(feature = "components")]
pub mod sheets;
#[cfg(feature = "components")]
pub mod shell;
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

#[cfg(test)]
mod testing;

#[cfg(feature = "components")]
pub use button::Button;
#[cfg(feature = "components")]
pub use carousel::{window_start, Carousel};
#[cfg(feature = "charts")]
pub use charts::{
    bar_max, fmt_num, heat_color, heat_color_with, marker_points, nice_ticks, norm, pie_fractions,
    pie_hit, polar, smooth_points, BarChart, Gauge, Heatmap, Legend, LineChart, Marker, PieChart,
    RadarChart, ScatterPlot, Series, Sparkline,
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
    overlay_lines, stack_cumulative, Candle, CandleStyle, Candlestick, Overlay, Pane,
    StackedAreaChart, TradeMarker,
};
#[cfg(feature = "components")]
pub use command::{fuzzy_filter, Combobox, Command, CommandPalette};
#[cfg(feature = "components")]
pub use controls::{
    tick_fractions, BubblePlacement, Checkbox, Direction, ProgressBar, Radio, Slider, Toggle,
};
#[cfg(feature = "components")]
pub use count::CountUp;
#[cfg(feature = "components")]
pub use data::{Avatar, Badge, Chip, Kbd, Table};
#[cfg(feature = "charts")]
pub use dodai_charts_core::gantt::Task as GanttTask;
#[cfg(feature = "charts")]
pub use dodai_charts_core::Gradient;
#[cfg(feature = "shaders")]
pub use dodai_shaders;
#[cfg(feature = "components")]
pub use extras::{
    step_value, BottomAppBar, ColorPicker, EmptyState, FileDropZone, NumberStepper, Rating,
    TagField, Timeline, TimelineEvent, Toolbar,
};
#[cfg(feature = "components")]
pub use feedback::{Alert, IndeterminateBar, Skeleton, Spinner, Toast};
pub use icons::{Icon, IconName};
#[cfg(feature = "components")]
pub use knob::Knob;
#[cfg(feature = "components")]
pub use layout::{clamp_ratio, visible_rows, SplitPane, TreeItem, TreeRow, TreeView};
#[cfg(feature = "components")]
pub use list::{Divider, ListItem};
#[cfg(feature = "components")]
pub use menu::{Menu, MenuItem};
pub use motion::{
    entry as entry_animation, grow, lerp, stagger_window, Curve, Easing, EntryKind, EntryScope,
    EntrySet, Motion, Tween,
};
#[cfg(feature = "components")]
pub use nav::{Accordion, Breadcrumb, Pagination, Segmented, Stepper};
#[cfg(feature = "components")]
pub use overlay::{Dialog, DialogCard};
pub use paint::Playback;
pub use paint::{shaped, ShapeStyle};
pub use panel::{DodaiTooltip, InputBox, Panel};
#[cfg(feature = "components")]
pub use picker::{
    days_in_month, month_grid, shift_month, step_hour, step_minute, to_12h, weekday, Date,
    DatePicker, TimePicker,
};
#[cfg(feature = "components")]
pub use range::{nearest_thumb, range_update, RangeSlider};
pub use scanlines::Scanlines;
#[cfg(feature = "components")]
pub use search::{count_label, highlight_runs, IconBadge, RichTooltip, SearchView};
#[cfg(feature = "components")]
pub use select::{family_title, DesignSystemSelector, Select, SelectOption, ThemeSelector};
#[cfg(feature = "shaders")]
pub use shader_view::{PaletteSource, ShaderView};
#[cfg(feature = "components")]
pub use sheets::{sheet_style, BottomSheet, FullScreenDialog, SheetEdge, SideSheet, Snackbar};
#[cfg(feature = "components")]
pub use shell::{is_wide, ResponsiveShell};
#[cfg(feature = "components")]
pub use surfaces::{
    card_style, AppBarSize, Card, CardVariant, Fab, FabSize, IconButton, NavItem, NavLayout,
    Navigation, SearchBar, TopAppBar,
};
#[cfg(feature = "components")]
pub use switcher::{set_tree_expanded, tree_expanded, DesignSystemTree, ThemeSwitcherBar};
#[cfg(feature = "components")]
pub use tabs::Tabs;
#[cfg(feature = "components")]
pub use tag::Tag;
pub use text::{Heading, Label, Mono, Muted};
#[cfg(feature = "components")]
pub use text_input::TextInput;

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
    pub fn theme(&self) -> &Theme {
        self.switcher.current()
    }

    pub fn switcher(&self) -> &ThemeSwitcher {
        &self.switcher
    }

    /// Hsla for a status tone.
    pub fn tone(&self, tone: Tone) -> Hsla {
        hsla(self.theme().palette.tone(tone))
    }
}

/// Install the global theme (first theme = `initial`, or the default list head).
pub fn install(cx: &mut App, initial: Theme) {
    let mut switcher = ThemeSwitcher::new();
    if !switcher.select(initial.id()) {
        let mut all = switcher.themes().to_vec();
        all.insert(0, initial);
        switcher = ThemeSwitcher::with_themes(all);
    }
    cx.set_global(DodaiTheme { switcher });
}

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
    fn glow_intensity_applies_to_all_themes() {
        let mut sw = ThemeSwitcher::new();
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
