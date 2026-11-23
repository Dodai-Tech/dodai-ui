//! Behaviour primitives: entry animation, pointer state, lazy sections, live samples, value
//! following, overscroll, snap stacks and scroll-spy; plus the knob, range slider and axis
//! gizmo. On the web the browser does most of the work, so these are thin wrappers over CSS and
//! the shared rules of `dodai-core`.

use super::kit::KitBox;
use dodai_core::motion::EntryKind;
use dodai_core::stream::RingBuffer;
use dodai_core::{spy, Kit, Theme};
use leptos::prelude::*;
use std::sync::{Arc, Mutex};

fn theme_or(theme: Option<Theme>) -> Option<Theme> {
    theme.or_else(|| use_context::<crate::GlassContext>().map(|g| g.theme))
}

/// A rotary knob drawn as inline SVG; `value` within `min..=max`. Needs a theme (given, or from
/// the provider).
#[component]
pub fn Knob(
    value: f32,
    #[prop(default = 0.0)] min: f32,
    #[prop(default = 1.0)] max: f32,
    #[prop(default = 72.0)] size: f32,
    #[prop(optional)] disabled: bool,
    #[prop(optional)] theme: Option<Theme>,
) -> impl IntoView {
    let theme = theme_or(theme)?;
    let frac = (value - min) / (max - min).max(f32::EPSILON);
    let svg = dodai_topcoat::html::knob_svg(&theme, size, frac, disabled);
    Some(view! {
        <svg class="dodai-knob" width=size.to_string() height=size.to_string() viewBox=format!("0 0 {size} {size}")
            role="slider" aria-valuemin=min.to_string() aria-valuemax=max.to_string() aria-valuenow=value.to_string()
            aria-disabled=disabled.to_string() style="user-select:none" inner_html=svg></svg>
    })
}

/// Two thumbs on one track picking a span; they never cross.
#[component]
pub fn RangeSlider(
    lo: f32,
    hi: f32,
    #[prop(default = 0.0)] min: f32,
    #[prop(default = 1.0)] max: f32,
    #[prop(default = 0.01)] step: f32,
    #[prop(optional)] vertical: bool,
    #[prop(default = 120)] length: u32,
) -> impl IntoView {
    let (lo, hi) = (lo.min(hi), hi.max(lo));
    let span = (max - min).max(f32::EPSILON);
    let pct = |v: f32| ((v - min) / span).clamp(0.0, 1.0) * 100.0;
    let (class, style, thumb) = if vertical {
        (
            "dodai-range dodai-range--v",
            format!("--a:{}%;--b:{}%;height:{length}px", pct(lo), pct(hi)),
            "dodai-slider dodai-slider--v",
        )
    } else {
        (
            "dodai-range",
            format!("--a:{}%;--b:{}%", pct(lo), pct(hi)),
            "dodai-slider",
        )
    };
    view! {
        <div class=class style=style>
            <input class=thumb type="range" min=min.to_string() max=max.to_string() step=step.to_string() value=lo.to_string() aria-label="From" />
            <input class=thumb type="range" min=min.to_string() max=max.to_string() step=step.to_string() value=hi.to_string() aria-label="To" />
        </div>
    }
}

/// The orientation gizmo of a 3D view as inline SVG: stems and balls of the axes for the orbit
/// camera at `yaw` and `pitch` (radians).
#[component]
pub fn AxisGizmo(
    yaw: f32,
    pitch: f32,
    #[prop(default = 130.0)] size: f32,
    #[prop(optional)] theme: Option<Theme>,
) -> impl IntoView {
    let theme = theme_or(theme)?;
    let svg = dodai_topcoat::html::gizmo_svg(&theme, size, yaw, pitch);
    Some(view! {
        <svg class="dodai-gizmo" width=size.to_string() height=size.to_string() viewBox=format!("0 0 {size} {size}") inner_html=svg></svg>
    })
}

fn entry_frames(kind: EntryKind) -> Option<(&'static str, &'static str)> {
    let to = "opacity:1;transform:none";
    match kind {
        EntryKind::None => None,
        EntryKind::Fade => Some(("opacity:0", "opacity:1")),
        EntryKind::Rise => Some(("opacity:0;transform:translateY(14px)", to)),
        EntryKind::Drop => Some(("opacity:0;transform:translateY(-14px)", to)),
        EntryKind::SlideLeft => Some(("opacity:0;transform:translateX(14px)", to)),
        EntryKind::SlideRight => Some(("opacity:0;transform:translateX(-14px)", to)),
    }
}

/// Every direct child animates in with `kind` when it first appears, staggered by `stagger_ms`.
/// Duration and easing are the design's motion tokens.
#[component]
pub fn EntryScope(
    #[prop(optional)] kind: EntryKind,
    #[prop(optional)] stagger_ms: u32,
    children: Children,
) -> impl IntoView {
    let Some((from, to)) = entry_frames(kind) else {
        return view! { <div style="display:contents">{children()}</div> }.into_any();
    };
    let css = format!(
        "@keyframes dodai-entry{{from{{{from}}}to{{{to}}}}}\
         .dodai-entry>*{{animation:dodai-entry var(--dodai-motion-duration,160ms) var(--dodai-motion-easing,ease) both;animation-delay:calc(var(--i,0) * {stagger_ms}ms)}}\
         @media (prefers-reduced-motion:reduce){{.dodai-entry>*{{animation:none}}}}"
    );
    view! {
        <style>{css}</style>
        <div class="dodai-entry" style="display:contents">{children()}</div>
    }
    .into_any()
}

/// Exposes hover and press of its children as `data-hovered` / `data-pressed` for a stylesheet.
#[component]
pub fn Interact(
    #[prop(optional)] hovered: bool,
    #[prop(optional)] pressed: bool,
    children: Children,
) -> impl IntoView {
    view! {
        <div style="display:contents" data-hovered=hovered.to_string() data-pressed=pressed.to_string()>{children()}</div>
    }
}

/// Lays out and paints a block only while it is near the window (`content-visibility:auto`),
/// holding its place with a spacer of `height` otherwise.
#[component]
pub fn Lazy(#[prop(default = 400)] height: u32, children: Children) -> impl IntoView {
    view! { <div style=format!("content-visibility:auto;contain-intrinsic-size:auto {height}px")>{children()}</div> }
}

/// A fixed-size window of recent samples. Clones share the same data.
#[derive(Clone)]
pub struct Live(Arc<Mutex<RingBuffer>>);

impl PartialEq for Live {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Live {
    /// A window of `capacity` samples, all zero.
    pub fn new(capacity: usize) -> Self {
        Self(Arc::new(Mutex::new(RingBuffer::filled(capacity, 0.0))))
    }
    /// Append a sample; the oldest drops out.
    pub fn push(&self, v: f32) {
        if let Ok(mut b) = self.0.lock() {
            b.push(v);
        }
    }
    /// The window, oldest first.
    pub fn samples(&self) -> Vec<f32> {
        self.0.lock().map(|b| b.to_vec()).unwrap_or_default()
    }
    /// How many samples were ever pushed.
    pub fn pushed(&self) -> u64 {
        self.0.lock().map(|b| b.pushed()).unwrap_or(0)
    }
}

/// Eases `--follow` to `value` with the design's motion tokens, so children can draw from
/// `var(--follow)` and glide to each change.
#[component]
pub fn Follow(value: f32, children: Children) -> impl IntoView {
    view! {
        <style>"@property --follow{syntax:'<number>';inherits:true;initial-value:0}"</style>
        <div style=format!("--follow:{value};transition:--follow var(--dodai-motion-duration,160ms) var(--dodai-motion-easing,ease);display:contents")>
            {children()}
        </div>
    }
}

/// How a scroll container reacts when pushed past its end.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum OverscrollFeel {
    /// The browser's own stretch or bounce, kept inside this container.
    #[default]
    Contain,
    /// The push scrolls the page behind once the container is at its end.
    Chain,
    /// No bounce and no chaining.
    None,
}

/// A scroll container with a chosen end-of-scroll feel (`overscroll-behavior`).
#[component]
pub fn Overscroll(
    #[prop(optional)] feel: OverscrollFeel,
    #[prop(default = "100%".to_string(), into)] height: String,
    children: Children,
) -> impl IntoView {
    let b = match feel {
        OverscrollFeel::Contain => "contain",
        OverscrollFeel::Chain => "auto",
        OverscrollFeel::None => "none",
    };
    view! { <div style=format!("height:{height};overflow:auto;overscroll-behavior:{b}")>{children()}</div> }
}

/// Pages stacked vertically; a drag or wheel snaps the nearest one into place.
#[component]
pub fn SnapStack(
    #[prop(default = "100%".to_string(), into)] height: String,
    children: Children,
) -> impl IntoView {
    view! {
        <style>".dodai-snap>*{scroll-snap-align:start;scroll-snap-stop:always;min-height:100%}"</style>
        <div class="dodai-snap" style=format!("height:{height};overflow-y:auto;scroll-snap-type:y mandatory;overscroll-behavior:contain")>{children()}</div>
    }
}

/// A sidebar of sections that highlights the one in view, by the rule every toolkit shares
/// ([`dodai_core::spy::pick`]). `sections` are `(label, top)`, `top` being the distance from
/// the window top in px; the caller measures them.
#[component]
pub fn ScrollSpy(
    sections: Vec<(String, f32)>,
    #[prop(optional)] at_end: bool,
    #[prop(optional)] pinned: Option<usize>,
    #[prop(default = 24.0)] slack: f32,
) -> impl IntoView {
    let tops: Vec<(usize, f32)> = sections
        .iter()
        .enumerate()
        .map(|(i, (_, t))| (i, *t))
        .collect();
    let current = spy::pick(&tops, pinned, 0.0, at_end, slack);
    let rows = sections
        .into_iter()
        .enumerate()
        .map(|(i, (label, _))| view! {
            <KitBox kit=Kit::Row { selected: current == Some(i) } pressable=true style="width:100%;justify-content:flex-start;">{label}</KitBox>
        })
        .collect_view();
    view! { <nav style="display:flex;flex-direction:column;gap:2px">{rows}</nav> }
}
