//! Behaviour primitives: entry animation, pointer state, lazy sections, live samples, value
//! following, overscroll, snap stacks and scroll-spy. On the web the browser does most of the
//! work, so these are thin wrappers over CSS (`content-visibility`, scroll snapping,
//! `overscroll-behavior`, registered custom properties) and the shared rules of `dodai-core`.

use dioxus::prelude::*;
use dodai_core::motion::EntryKind;
use dodai_core::spy;
use dodai_core::stream::RingBuffer;
use std::sync::{Arc, Mutex};

fn entry_frames(kind: EntryKind) -> Option<(&'static str, &'static str)> {
    match kind {
        EntryKind::None => None,
        EntryKind::Fade => Some(("opacity:0", "opacity:1")),
        EntryKind::Rise => Some((
            "opacity:0;transform:translateY(14px)",
            "opacity:1;transform:none",
        )),
        EntryKind::Drop => Some((
            "opacity:0;transform:translateY(-14px)",
            "opacity:1;transform:none",
        )),
        EntryKind::SlideLeft => Some((
            "opacity:0;transform:translateX(14px)",
            "opacity:1;transform:none",
        )),
        EntryKind::SlideRight => Some((
            "opacity:0;transform:translateX(-14px)",
            "opacity:1;transform:none",
        )),
    }
}

/// Properties of [`EntryScope`].
#[derive(Props, Clone, PartialEq)]
pub struct EntryScopeProps {
    #[props(default)]
    pub kind: EntryKind,
    /// Delay between children in ms (0 = all at once).
    #[props(default)]
    pub stagger_ms: u32,
    pub children: Element,
}

/// Every direct child animates in with `kind` when it first appears, staggered by `stagger_ms`.
/// The duration and easing are the design's motion tokens.
#[component]
pub fn EntryScope(props: EntryScopeProps) -> Element {
    let Some((from, to)) = entry_frames(props.kind) else {
        return rsx! { div { style: "display:contents", {props.children} } };
    };
    let step = props.stagger_ms;
    let css = format!(
        "@keyframes dodai-entry{{from{{{from}}}to{{{to}}}}}\
         .dodai-entry>*{{animation:dodai-entry var(--dodai-motion-duration,160ms) var(--dodai-motion-easing,ease) both;animation-delay:calc(var(--i,0) * {step}ms)}}\
         @media (prefers-reduced-motion:reduce){{.dodai-entry>*{{animation:none}}}}"
    );
    rsx! {
        style { "{css}" }
        div { class: "dodai-entry", style: "display:contents", {props.children} }
    }
}

/// Properties of [`Interact`].
#[derive(Props, Clone, PartialEq)]
pub struct InteractProps {
    /// Called with `(hovered, pressed)` whenever either changes.
    #[props(default)]
    pub onchange: EventHandler<(bool, bool)>,
    pub children: Element,
}

/// Tracks hover and press of its children and exposes them as `data-hovered` / `data-pressed`
/// attributes, so a stylesheet (or the caller, through `onchange`) can restyle.
#[component]
pub fn Interact(props: InteractProps) -> Element {
    let mut state = use_signal(|| (false, false));
    let on = props.onchange;
    let mut set = move |s: (bool, bool)| {
        if state() != s {
            state.set(s);
            on.call(s);
        }
    };
    let (h, p) = state();
    rsx! {
        div { style: "display:contents", "data-hovered": "{h}", "data-pressed": "{p}",
            onmouseenter: move |_| set((true, state().1)),
            onmouseleave: move |_| set((false, false)),
            onmousedown: move |_| set((state().0, true)),
            onmouseup: move |_| set((state().0, false)),
            {props.children}
        }
    }
}

/// Properties of [`Lazy`].
#[derive(Props, Clone, PartialEq)]
pub struct LazyProps {
    /// Height in px reserved while the block is off screen.
    #[props(default = 400)]
    pub height: u32,
    pub children: Element,
}

/// Lays out and paints a block only while it is near the window (`content-visibility:auto`),
/// holding its place with a spacer of `height` otherwise.
#[component]
pub fn Lazy(props: LazyProps) -> Element {
    rsx! { div { style: "content-visibility:auto;contain-intrinsic-size:auto {props.height}px", {props.children} } }
}

/// A fixed-size window of recent samples. Clones share the same data, so a source can push
/// from anywhere while a chart reads.
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
    /// How many samples were ever pushed (a change counter for redraws).
    pub fn pushed(&self) -> u64 {
        self.0.lock().map(|b| b.pushed()).unwrap_or(0)
    }
}

/// Properties of [`Follow`].
#[derive(Props, Clone, PartialEq)]
pub struct FollowProps {
    pub value: f32,
    pub children: Element,
}

/// Eases `--follow` to `value` with the design's motion tokens, so children can draw from
/// `var(--follow)` (a needle's `rotate(calc(var(--follow) * 1deg))`) and glide to each change.
#[component]
pub fn Follow(props: FollowProps) -> Element {
    rsx! {
        style { "@property --follow{{syntax:'<number>';inherits:true;initial-value:0}}" }
        div { style: "--follow:{props.value};transition:--follow var(--dodai-motion-duration,160ms) var(--dodai-motion-easing,ease);display:contents",
            {props.children}
        }
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

/// Properties of [`Overscroll`].
#[derive(Props, Clone, PartialEq)]
pub struct OverscrollProps {
    #[props(default)]
    pub feel: OverscrollFeel,
    #[props(default = "100%".to_string())]
    pub height: String,
    pub children: Element,
}

/// A scroll container with a chosen end-of-scroll feel (`overscroll-behavior`).
#[component]
pub fn Overscroll(props: OverscrollProps) -> Element {
    let b = match props.feel {
        OverscrollFeel::Contain => "contain",
        OverscrollFeel::Chain => "auto",
        OverscrollFeel::None => "none",
    };
    rsx! { div { style: "height:{props.height};overflow:auto;overscroll-behavior:{b}", {props.children} } }
}

/// Properties of [`SnapStack`].
#[derive(Props, Clone, PartialEq)]
pub struct SnapStackProps {
    #[props(default = "100%".to_string())]
    pub height: String,
    /// One child per page; each fills the stack.
    pub children: Element,
}

/// Pages stacked vertically; a drag or wheel snaps the nearest one into place.
#[component]
pub fn SnapStack(props: SnapStackProps) -> Element {
    rsx! {
        style { ".dodai-snap>*{{scroll-snap-align:start;scroll-snap-stop:always;min-height:100%}}" }
        div { class: "dodai-snap", style: "height:{props.height};overflow-y:auto;scroll-snap-type:y mandatory;overscroll-behavior:contain", {props.children} }
    }
}

/// Properties of [`ScrollSpy`].
#[derive(Props, Clone, PartialEq)]
pub struct ScrollSpyProps {
    /// `(label, top)` per section in document order; `top` is the section's distance from the
    /// window top in px.
    pub sections: Vec<(String, f32)>,
    /// Whether the scroll range is used up.
    #[props(default)]
    pub at_end: bool,
    /// A section the user clicked, which stays current until the page scrolls.
    #[props(default)]
    pub pinned: Option<usize>,
    #[props(default = 24.0)]
    pub slack: f32,
    #[props(default)]
    pub onpick: EventHandler<usize>,
}

/// A sidebar of sections that highlights the one in view, by the rule every toolkit shares
/// ([`dodai_core::spy::pick`]). The caller measures the tops.
#[component]
pub fn ScrollSpy(props: ScrollSpyProps) -> Element {
    let tops: Vec<(usize, f32)> = props
        .sections
        .iter()
        .enumerate()
        .map(|(i, (_, t))| (i, *t))
        .collect();
    let current = spy::pick(&tops, props.pinned, 0.0, props.at_end, props.slack);
    let on = props.onpick;
    rsx! {
        nav { style: "display:flex;flex-direction:column;gap:2px",
            for (i, (label, _)) in props.sections.iter().enumerate() {
                crate::KitBox { key: "{i}", kit: dodai_core::Kit::Row { selected: current == Some(i) },
                    style: "width:100%;justify-content:flex-start".to_string(), onclick: move |_| on.call(i), "{label}" }
            }
        }
    }
}
