//! Behaviour primitives: entry animation, pointer state, lazy sections, live samples, value
//! following, overscroll, snap stacks and scroll-spy; plus the knob, range slider and axis
//! gizmo. On the web the browser does most of the work, so these are thin wrappers over CSS and
//! the shared rules of `dodai-ui-core`.

use super::kit::KitBox;
use crate::dom::input_value;
use crate::use_theme;
use dodai_ui_core::motion::EntryKind;
use dodai_ui_core::orientation::{gizmo_click, orbit, DragTracker};
use dodai_ui_core::stream::RingBuffer;
use dodai_ui_core::{knob_value, spy, Kit, Theme};
use dodai_ui_topcoat::html::{gizmo_svg, knob_svg};
use std::cell::RefCell;
use std::rc::Rc;
use yew::prelude::*;

/// The given theme, else the provider's. Call `use_theme()` first: hooks can't run in a closure.
fn theme_or(theme: &Option<Theme>, ctx: Option<crate::ThemeContext>) -> Option<Theme> {
    theme.clone().or_else(|| ctx.map(|c| c.theme))
}

#[derive(Properties, PartialEq)]
pub struct KnobProps {
    pub value: f32,
    #[prop_or_default]
    pub min: f32,
    #[prop_or(1.0)]
    pub max: f32,
    #[prop_or_default]
    pub step: Option<f32>,
    #[prop_or(72.0)]
    pub size: f32,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub theme: Option<Theme>,
    #[prop_or_default]
    pub onchange: Callback<f32>,
}

/// A rotary knob drawn as inline SVG; drag up or down, or use the wheel. Needs a theme (given,
/// or from the provider).
#[function_component]
pub fn Knob(props: &KnobProps) -> Html {
    let anchor = use_state(|| None::<(f32, f32, f32)>);
    let ctx = use_theme();
    let Some(theme) = theme_or(&props.theme, ctx) else {
        return html! {};
    };
    let (min, max, step, value, size, disabled) = (
        props.min,
        props.max,
        props.step,
        props.value,
        props.size,
        props.disabled,
    );
    let frac = (value - min) / (max - min).max(f32::EPSILON);
    let svg = knob_svg(&theme, size, frac, disabled);
    let (down, mv, up, leave, wheel) = (
        anchor.clone(),
        anchor.clone(),
        anchor.clone(),
        anchor.clone(),
        props.onchange.clone(),
    );
    let h = props.onchange.clone();
    let dim = if disabled {
        "opacity:.45;"
    } else {
        "cursor:ns-resize;"
    };
    html! {
        <svg class="dodai-knob" width={size.to_string()} height={size.to_string()} viewBox={format!("0 0 {size} {size}")}
            style={format!("{dim}touch-action:none;user-select:none")} role="slider"
            aria-valuemin={min.to_string()} aria-valuemax={max.to_string()} aria-valuenow={value.to_string()} aria-disabled={disabled.to_string()}
            onmousedown={Callback::from(move |e: MouseEvent| if !disabled { down.set(Some((e.client_x() as f32, e.client_y() as f32, value))); })}
            onmousemove={Callback::from(move |e: MouseEvent| if let Some((ax, ay, v0)) = *mv {
                h.emit(knob_value(v0, e.client_x() as f32 - ax, e.client_y() as f32 - ay, min, max, step));
            })}
            onmouseup={Callback::from(move |_| up.set(None))}
            onmouseleave={Callback::from(move |_| leave.set(None))}
            onwheel={Callback::from(move |e: WheelEvent| if !disabled {
                wheel.emit(knob_value(value, 0.0, (e.delta_y() as f32).signum() * 8.0, min, max, step));
            })}>
            { Html::from_html_unchecked(AttrValue::from(svg)) }
        </svg>
    }
}

#[derive(Properties, PartialEq)]
pub struct RangeSliderProps {
    /// One thumb.
    pub lo: f32,
    /// The other thumb.
    pub hi: f32,
    #[prop_or_default]
    pub min: f32,
    #[prop_or(1.0)]
    pub max: f32,
    #[prop_or(0.01)]
    pub step: f32,
    /// Runs bottom to top, `length` px tall.
    #[prop_or_default]
    pub vertical: bool,
    #[prop_or(120)]
    pub length: u32,
    /// The new `(low, high)`, always in order.
    #[prop_or_default]
    pub onchange: Callback<(f32, f32)>,
}

/// Two thumbs on one track picking a span; they never cross.
#[function_component]
pub fn RangeSlider(props: &RangeSliderProps) -> Html {
    let (lo, hi) = (props.lo.min(props.hi), props.hi.max(props.lo));
    let span = (props.max - props.min).max(f32::EPSILON);
    let pct = |v: f32| ((v - props.min) / span).clamp(0.0, 1.0) * 100.0;
    let (class, style, thumb) = if props.vertical {
        (
            "dodai-range dodai-range--v",
            format!(
                "--a:{}%;--b:{}%;height:{}px",
                pct(lo),
                pct(hi),
                props.length
            ),
            "dodai-slider dodai-slider--v",
        )
    } else {
        (
            "dodai-range",
            format!("--a:{}%;--b:{}%", pct(lo), pct(hi)),
            "dodai-slider",
        )
    };
    let (a, b) = (props.onchange.clone(), props.onchange.clone());
    html! {
        <div {class} {style}>
            <input class={thumb} type="range" min={props.min.to_string()} max={props.max.to_string()} step={props.step.to_string()} value={lo.to_string()} aria-label="From"
                oninput={Callback::from(move |e: InputEvent| if let Ok(v) = input_value(&e).parse::<f32>() { a.emit((v.min(hi), hi)) })} />
            <input class={thumb} type="range" min={props.min.to_string()} max={props.max.to_string()} step={props.step.to_string()} value={hi.to_string()} aria-label="To"
                oninput={Callback::from(move |e: InputEvent| if let Ok(v) = input_value(&e).parse::<f32>() { b.emit((lo, v.max(lo))) })} />
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct AxisGizmoProps {
    pub yaw: f32,
    pub pitch: f32,
    #[prop_or(130.0)]
    pub size: f32,
    #[prop_or_default]
    pub theme: Option<Theme>,
    /// The new `(yaw, pitch)` from a drag, an axis press or home.
    #[prop_or_default]
    pub onchange: Callback<(f32, f32)>,
}

/// The orientation gizmo of a 3D view as inline SVG: click a ball to look along its axis, home
/// to go back, drag to orbit.
#[function_component]
pub fn AxisGizmo(props: &AxisGizmoProps) -> Html {
    let press = use_state(|| None::<(f32, f32)>);
    let drag = use_mut_ref(DragTracker::default);
    let ctx = use_theme();
    let Some(theme) = theme_or(&props.theme, ctx) else {
        return html! {};
    };
    let (yaw, pitch, size) = (props.yaw, props.pitch, props.size);
    let svg = gizmo_svg(&theme, size, yaw, pitch);
    let local = |e: &MouseEvent| (e.offset_x() as f32, e.offset_y() as f32);
    let (p1, d1) = (press.clone(), drag.clone());
    let (p2, d2, h2) = (press.clone(), drag.clone(), props.onchange.clone());
    let (p3, d3, h3) = (press.clone(), drag.clone(), props.onchange.clone());
    let p4 = press.clone();
    html! {
        <svg class="dodai-gizmo" width={size.to_string()} height={size.to_string()} viewBox={format!("0 0 {size} {size}")}
            style="cursor:pointer;touch-action:none;user-select:none"
            onmousedown={Callback::from(move |e: MouseEvent| { p1.set(Some(local(&e))); d1.borrow_mut().press(); })}
            onmousemove={Callback::from(move |e: MouseEvent| if let Some((x, y)) = *p2 {
                let (qx, qy) = local(&e);
                let (dx, dy) = (qx - x, qy - y);
                d2.borrow_mut().add(dx, dy);
                p2.set(Some((qx, qy)));
                if d2.borrow().is_drag() { h2.emit(orbit(yaw, pitch, dx, dy)); }
            })}
            onmouseup={Callback::from(move |e: MouseEvent| {
                if p3.is_some() && !d3.borrow().is_drag() {
                    let (qx, qy) = local(&e);
                    if let Some(cam) = gizmo_click(size, yaw, pitch, [qx, qy]) { h3.emit(cam); }
                }
                p3.set(None);
            })}
            onmouseleave={Callback::from(move |_| p4.set(None))}>
            { Html::from_html_unchecked(AttrValue::from(svg)) }
        </svg>
    }
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

#[derive(Properties, PartialEq)]
pub struct EntryScopeProps {
    #[prop_or_default]
    pub kind: EntryKind,
    /// Delay between children in ms (0 = all at once).
    #[prop_or_default]
    pub stagger_ms: u32,
    #[prop_or_default]
    pub children: Html,
}

/// Every direct child animates in with `kind` when it first appears, staggered by `stagger_ms`.
/// Duration and easing are the design's motion tokens.
#[function_component]
pub fn EntryScope(props: &EntryScopeProps) -> Html {
    let Some((from, to)) = entry_frames(props.kind) else {
        return html! { <div style="display:contents">{ props.children.clone() }</div> };
    };
    let step = props.stagger_ms;
    let css = format!(
        "@keyframes dodai-entry{{from{{{from}}}to{{{to}}}}}\
         .dodai-entry>*{{animation:dodai-entry var(--dodai-motion-duration,160ms) var(--dodai-motion-easing,ease) both;animation-delay:calc(var(--i,0) * {step}ms)}}\
         @media (prefers-reduced-motion:reduce){{.dodai-entry>*{{animation:none}}}}"
    );
    html! {
        <>
            <style>{ css }</style>
            <div class="dodai-entry" style="display:contents">{ props.children.clone() }</div>
        </>
    }
}

#[derive(Properties, PartialEq)]
pub struct InteractProps {
    /// Called with `(hovered, pressed)` whenever either changes.
    #[prop_or_default]
    pub onchange: Callback<(bool, bool)>,
    #[prop_or_default]
    pub children: Html,
}

/// Tracks hover and press of its children and exposes them as `data-hovered` / `data-pressed`.
#[function_component]
pub fn Interact(props: &InteractProps) -> Html {
    let state = use_state(|| (false, false));
    let set = |next: fn((bool, bool)) -> (bool, bool)| {
        let (state, cb) = (state.clone(), props.onchange.clone());
        Callback::from(move |_: MouseEvent| {
            let n = next(*state);
            if n != *state {
                state.set(n);
                cb.emit(n);
            }
        })
    };
    html! {
        <div style="display:contents" data-hovered={state.0.to_string()} data-pressed={state.1.to_string()}
            onmouseenter={set(|s| (true, s.1))} onmouseleave={set(|_| (false, false))}
            onmousedown={set(|s| (s.0, true))} onmouseup={set(|s| (s.0, false))}>
            { props.children.clone() }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct LazyProps {
    /// Height in px reserved while the block is off screen.
    #[prop_or(400)]
    pub height: u32,
    #[prop_or_default]
    pub children: Html,
}

/// Lays out and paints a block only while it is near the window (`content-visibility:auto`),
/// holding its place with a spacer of `height` otherwise.
#[function_component]
pub fn Lazy(props: &LazyProps) -> Html {
    html! { <div style={format!("content-visibility:auto;contain-intrinsic-size:auto {}px", props.height)}>{ props.children.clone() }</div> }
}

/// A fixed-size window of recent samples. Clones share the same data.
#[derive(Clone)]
pub struct Live(Rc<RefCell<RingBuffer>>);

impl PartialEq for Live {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Live {
    /// A window of `capacity` samples, all zero.
    pub fn new(capacity: usize) -> Self {
        Self(Rc::new(RefCell::new(RingBuffer::filled(capacity, 0.0))))
    }
    /// Append a sample; the oldest drops out.
    pub fn push(&self, v: f32) {
        self.0.borrow_mut().push(v);
    }
    /// The window, oldest first.
    pub fn samples(&self) -> Vec<f32> {
        self.0.borrow().to_vec()
    }
    /// How many samples were ever pushed.
    pub fn pushed(&self) -> u64 {
        self.0.borrow().pushed()
    }
}

#[derive(Properties, PartialEq)]
pub struct FollowProps {
    pub value: f32,
    #[prop_or_default]
    pub children: Html,
}

/// Eases `--follow` to `value` with the design's motion tokens, so children can draw from
/// `var(--follow)` and glide to each change.
#[function_component]
pub fn Follow(props: &FollowProps) -> Html {
    html! {
        <>
            <style>{ "@property --follow{syntax:'<number>';inherits:true;initial-value:0}" }</style>
            <div style={format!("--follow:{};transition:--follow var(--dodai-motion-duration,160ms) var(--dodai-motion-easing,ease);display:contents", props.value)}>
                { props.children.clone() }
            </div>
        </>
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

#[derive(Properties, PartialEq)]
pub struct OverscrollProps {
    #[prop_or_default]
    pub feel: OverscrollFeel,
    #[prop_or(AttrValue::Static("100%"))]
    pub height: AttrValue,
    #[prop_or_default]
    pub children: Html,
}

/// A scroll container with a chosen end-of-scroll feel (`overscroll-behavior`).
#[function_component]
pub fn Overscroll(props: &OverscrollProps) -> Html {
    let b = match props.feel {
        OverscrollFeel::Contain => "contain",
        OverscrollFeel::Chain => "auto",
        OverscrollFeel::None => "none",
    };
    html! { <div style={format!("height:{};overflow:auto;overscroll-behavior:{b}", props.height)}>{ props.children.clone() }</div> }
}

#[derive(Properties, PartialEq)]
pub struct SnapStackProps {
    #[prop_or(AttrValue::Static("100%"))]
    pub height: AttrValue,
    /// One child per page; each fills the stack.
    #[prop_or_default]
    pub children: Html,
}

/// Pages stacked vertically; a drag or wheel snaps the nearest one into place.
#[function_component]
pub fn SnapStack(props: &SnapStackProps) -> Html {
    html! {
        <>
            <style>{ ".dodai-snap>*{scroll-snap-align:start;scroll-snap-stop:always;min-height:100%}" }</style>
            <div class="dodai-snap" style={format!("height:{};overflow-y:auto;scroll-snap-type:y mandatory;overscroll-behavior:contain", props.height)}>{ props.children.clone() }</div>
        </>
    }
}

#[derive(Properties, PartialEq)]
pub struct ScrollSpyProps {
    /// `(label, top)` per section in document order; `top` is the distance from the window top in px.
    pub sections: Vec<(AttrValue, f32)>,
    /// Whether the scroll range is used up.
    #[prop_or_default]
    pub at_end: bool,
    /// A section the user clicked, which stays current until the page scrolls.
    #[prop_or_default]
    pub pinned: Option<usize>,
    #[prop_or(24.0)]
    pub slack: f32,
    #[prop_or_default]
    pub onpick: Callback<usize>,
}

/// A sidebar of sections that highlights the one in view, by the rule every toolkit shares
/// ([`dodai_ui_core::spy::pick`]). The caller measures the tops.
#[function_component]
pub fn ScrollSpy(props: &ScrollSpyProps) -> Html {
    let tops: Vec<(usize, f32)> = props
        .sections
        .iter()
        .enumerate()
        .map(|(i, (_, t))| (i, *t))
        .collect();
    let current = spy::pick(&tops, props.pinned, 0.0, props.at_end, props.slack);
    html! {
        <nav style="display:flex;flex-direction:column;gap:2px">
            { for props.sections.iter().enumerate().map(|(i, (label, _))| {
                let cb = props.onpick.clone();
                html! {
                    <KitBox kit={Kit::Row { selected: current == Some(i) }} pressable=true onclick={Callback::from(move |_| cb.emit(i))}
                        style="width:100%;justify-content:flex-start;">{ label.clone() }</KitBox>
                }
            }) }
        </nav>
    }
}
