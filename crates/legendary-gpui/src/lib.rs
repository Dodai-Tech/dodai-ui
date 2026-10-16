//! # legendary-gpui
//!
//! Turns a [`legendary_core::Theme`] into GPUI values: colours, a [`Global`]
//! theme store (`cx.legendary()`), and a [`LegendaryStyled`] extension for
//! any [`Styled`] element (`div().lg_panel(&theme)`, `.lg_button(...)`).
//!
//! ## Components
//!
//! Every component is a `RenderOnce` builder reading the active theme from the
//! global ([`install`] first): [`Button`], [`Panel`], [`Tag`], [`Tabs`],
//! [`Toggle`], [`Checkbox`], [`Radio`], [`ProgressBar`], [`Slider`], [`InputBox`],
//! [`ListItem`], [`Divider`], [`LegendaryTooltip`], [`Heading`]/[`Label`]/[`Muted`]/[`Mono`],
//! [`Scanlines`], [`ThemeSwitcherBar`]. Shapes (square / rounded / chamfered),
//! glow, hard shadow and uppercase labels all come from tokens; no component
//! looks at a theme id. See [`paint`] for the shaped-box primitive.
//!
//! ```no_run
//! use gpui::{div, prelude::*, App};
//! use legendary_core::{ButtonVariant, ControlState, Theme};
//! use legendary_gpui::{install, LegendaryStyled, ActiveLegendary};
//!
//! fn setup(cx: &mut App) {
//!     install(cx, Theme::default());
//!     let t = cx.legendary().theme().clone();
//!     let _btn = div().lg_button(&t, ButtonVariant::Primary, ControlState::Idle);
//! }
//! ```

pub mod button;
pub mod controls;
pub mod data;
pub mod feedback;
pub mod interact;
pub mod knob;
pub mod list;
pub mod logic;
pub mod menu;
pub mod nav;
pub mod overlay;
pub mod paint;
pub mod panel;
pub mod scanlines;
pub mod select;
pub mod surfaces;
pub mod switcher;
pub mod tabs;
pub mod tag;
pub mod text;
pub mod text_input;

#[cfg(test)]
mod testing;

pub use button::Button;
pub use controls::{Checkbox, Direction, ProgressBar, Radio, Slider, Toggle};
pub use data::{Avatar, Badge, Chip, Kbd, Table};
pub use feedback::{Alert, IndeterminateBar, Skeleton, Spinner, Toast};
pub use knob::Knob;
pub use list::{Divider, ListItem};
pub use menu::{Menu, MenuItem};
pub use nav::{Accordion, Breadcrumb, Pagination, Segmented, Stepper};
pub use overlay::{Dialog, DialogCard};
pub use paint::{shaped, ShapeStyle};
pub use panel::{InputBox, LegendaryTooltip, Panel};
pub use scanlines::Scanlines;
pub use select::{family_title, DesignSystemSelector, Select, SelectOption, ThemeSelector};
pub use surfaces::{
    card_style, AppBarSize, Card, CardVariant, Fab, FabSize, IconButton, NavItem, NavLayout,
    Navigation, SearchBar, TopAppBar,
};
pub use switcher::ThemeSwitcherBar;
pub use tabs::Tabs;
pub use tag::Tag;
pub use text::{Heading, Label, Mono, Muted};
pub use text_input::TextInput;

use gpui::{px, App, BoxShadow, Global, Hsla, Rgba, Styled};
use legendary_core::{ButtonVariant, Color, ControlState, Theme, ThemeSwitcher, Tone};

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
pub struct LegendaryTheme {
    switcher: ThemeSwitcher,
}

impl Global for LegendaryTheme {}

impl LegendaryTheme {
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
    cx.set_global(LegendaryTheme { switcher });
}

/// Clone of the active theme (components call this on render).
pub fn theme_of(cx: &App) -> Theme {
    cx.legendary().theme().clone()
}

/// Select theme by index (clamped) and repaint.
pub fn select_theme(cx: &mut App, index: usize) {
    cx.global_mut::<LegendaryTheme>()
        .switcher
        .select_index(index);
    cx.refresh_windows();
}

/// Current glow intensity of the active theme.
pub fn glow_intensity(cx: &App) -> f32 {
    cx.legendary().theme().effects.glow_intensity
}

/// Set glow intensity on **every** theme (so it survives switching) and repaint.
pub fn set_glow_intensity(cx: &mut App, intensity: f32) {
    let g = cx.global_mut::<LegendaryTheme>();
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

/// Whether the fine scanline texture is on for the active theme.
pub fn scanlines_enabled(cx: &App) -> bool {
    cx.legendary().theme().effects.scanlines
}

/// Turn the scanline texture (component bodies + [`Scanlines`] overlay) on/off
/// on **every** theme (so it survives switching) and repaint.
pub fn set_scanlines(cx: &mut App, on: bool) {
    let g = cx.global_mut::<LegendaryTheme>();
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
    cx.global_mut::<LegendaryTheme>().switcher.cycle_next();
    cx.refresh_windows();
}

/// Switch to the previous theme and repaint.
pub fn cycle_prev(cx: &mut App) {
    cx.global_mut::<LegendaryTheme>().switcher.cycle_prev();
    cx.refresh_windows();
}

/// `cx.legendary()` accessor.
pub trait ActiveLegendary {
    fn legendary(&self) -> &LegendaryTheme;
}

impl ActiveLegendary for App {
    fn legendary(&self) -> &LegendaryTheme {
        self.global::<LegendaryTheme>()
    }
}

/// Theme-aware styling for any GPUI [`Styled`] element.
pub trait LegendaryStyled: Styled + Sized {
    /// App background + default text colour.
    fn lg_background(self, t: &Theme) -> Self {
        self.bg(hsla(t.palette.background))
            .text_color(hsla(t.palette.text))
    }

    /// Bordered panel; glow shadow when `focused`.
    fn lg_panel(self, t: &Theme, focused: bool) -> Self {
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
    fn lg_button(self, t: &Theme, v: ButtonVariant, state: ControlState) -> Self {
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
    fn lg_input(self, t: &Theme, focused: bool) -> Self {
        let i = t.input(focused);
        self.bg(hsla(i.background))
            .border_2()
            .border_color(hsla(i.border))
            .text_color(hsla(i.text))
            .px(px(t.spacing.sm))
            .rounded(px(t.shape.radius))
    }
}

impl<T: Styled + Sized> LegendaryStyled for T {}

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
        let _ = gpui::div().lg_panel(&t, true).lg_button(
            &t,
            ButtonVariant::Outline,
            ControlState::Hovered,
        );
    }
}
