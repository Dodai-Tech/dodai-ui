//! A rotary knob drawn as inline SVG; geometry, drag maths and colours come from `dodai-core`.

use dioxus::prelude::*;
use dodai_core::{arc_points, knob_angle, knob_value, KnobGeometry, Theme};

use crate::ThemeHandle;

/// `M x y L x y …` through `pts`.
fn path(pts: &[(f32, f32)]) -> String {
    let mut d = String::new();
    for (i, (x, y)) in pts.iter().enumerate() {
        d.push_str(if i == 0 { "M" } else { "L" });
        d.push_str(&format!("{x:.2} {y:.2} "));
    }
    d
}

/// The SVG of a knob `size` px across, filled to `frac` (`0..=1`), coloured by `theme`.
pub fn knob_svg(theme: &Theme, size: f32, frac: f32, disabled: bool) -> String {
    let st = theme.knob(disabled);
    let g = KnobGeometry::new(size);
    let c = size / 2.0;
    let frac = frac.clamp(0.0, 1.0);
    let track = path(&arc_points(c, c, g.r_ring, 0.0, 1.0));
    let mut s = format!(
        "<path d=\"{track}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\"/>",
        st.track.to_css(),
        g.ring
    );
    if frac > 0.0 {
        let arc = path(&arc_points(c, c, g.r_ring, 0.0, frac));
        if let Some((col, r)) = st.glow {
            s.push_str(&format!(
                "<path d=\"{arc}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\" opacity=\"0.5\" style=\"filter:blur({}px)\"/>",
                col.to_css(),
                g.ring + r * 0.5,
                (r * 0.5).max(1.0)
            ));
        }
        s.push_str(&format!(
            "<path d=\"{arc}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\"/>",
            st.fill.to_css(),
            g.ring
        ));
    }
    s.push_str(&format!(
        "<circle cx=\"{c}\" cy=\"{c}\" r=\"{}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"{}\"/>",
        g.r_body,
        st.body.to_css(),
        st.body_edge.to_css(),
        st.border_width
    ));
    let th = knob_angle(frac);
    let (co, si) = (th.cos(), th.sin());
    s.push_str(&format!(
        "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"{}\" stroke-width=\"{}\"/>",
        c + co * g.r_body * 0.35,
        c + si * g.r_body * 0.35,
        c + co * g.r_body * 0.9,
        c + si * g.r_body * 0.9,
        st.tick.to_css(),
        g.tick
    ));
    s
}

/// Knob properties.
#[derive(Props, Clone, PartialEq)]
pub struct KnobProps {
    /// The value.
    pub value: f32,
    /// Lowest value.
    #[props(default)]
    pub min: f32,
    /// Highest value.
    #[props(default = 1.0)]
    pub max: f32,
    /// Snap to multiples of this from `min`.
    #[props(default)]
    pub step: Option<f32>,
    /// Diameter in px.
    #[props(default = 72.0)]
    pub size: f32,
    /// Dimmed and not draggable.
    #[props(default)]
    pub disabled: bool,
    /// Theme to draw with (default: the provider's).
    #[props(default)]
    pub theme: Option<Theme>,
    /// Called with the new value while dragging or scrolling.
    #[props(default)]
    pub onchange: EventHandler<f32>,
}

/// Rotary knob: drag up or right to increase, the wheel turns it too.
#[component]
pub fn Knob(props: KnobProps) -> Element {
    let Some(theme) = props
        .theme
        .clone()
        .or_else(|| try_use_context::<ThemeHandle>().map(|h| h.theme()))
    else {
        return rsx! {};
    };
    let mut anchor = use_signal(|| None::<(f32, f32, f32)>);
    let span = (props.max - props.min).max(f32::EPSILON);
    let frac = (props.value - props.min) / span;
    let svg = knob_svg(&theme, props.size, frac, props.disabled);
    let (min, max, step, value, size, disabled, h) = (
        props.min,
        props.max,
        props.step,
        props.value,
        props.size,
        props.disabled,
        props.onchange,
    );
    let dim = if disabled {
        "opacity:.45;"
    } else {
        "cursor:ns-resize;"
    };
    rsx! {
        svg { class: "dodai-knob", width: "{size}", height: "{size}", view_box: "0 0 {size} {size}",
            style: "{dim}touch-action:none;user-select:none",
            role: "slider", "aria-valuemin": "{min}", "aria-valuemax": "{max}", "aria-valuenow": "{value}",
            "aria-disabled": "{disabled}",
            onmousedown: move |e| if !disabled {
                let p = e.client_coordinates();
                anchor.set(Some((p.x as f32, p.y as f32, value)));
            },
            onmousemove: move |e| if let Some((ax, ay, v0)) = anchor() {
                let p = e.client_coordinates();
                h.call(knob_value(v0, p.x as f32 - ax, p.y as f32 - ay, min, max, step));
            },
            onmouseup: move |_| anchor.set(None),
            onmouseleave: move |_| anchor.set(None),
            onwheel: move |e| if !disabled {
                let dy = e.delta().strip_units().y as f32;
                h.call(knob_value(value, 0.0, dy.signum() * 8.0, min, max, step));
            },
            dangerous_inner_html: "{svg}"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dodai_core::themes;

    #[test]
    fn the_svg_has_track_value_arc_body_and_tick() {
        let t = themes::all().remove(0);
        let s = knob_svg(&t, 72.0, 0.5, false);
        assert!(s.matches("<path").count() >= 2);
        assert!(s.contains("<circle") && s.contains("<line"));
    }

    #[test]
    fn an_empty_knob_draws_no_value_arc_and_a_disabled_one_no_glow() {
        for t in themes::all() {
            let empty = knob_svg(&t, 72.0, 0.0, false);
            assert_eq!(empty.matches("<path").count(), 1, "{}", t.id());
            assert!(
                !knob_svg(&t, 72.0, 1.0, true).contains("blur"),
                "{}",
                t.id()
            );
        }
    }

    #[test]
    fn the_fraction_is_clamped() {
        let t = themes::all().remove(0);
        assert_eq!(
            knob_svg(&t, 72.0, 7.0, false),
            knob_svg(&t, 72.0, 1.0, false)
        );
        assert_eq!(
            knob_svg(&t, 72.0, -1.0, false),
            knob_svg(&t, 72.0, 0.0, false)
        );
    }

    #[test]
    fn it_renders_in_a_provider() {
        fn app() -> Element {
            rsx! { crate::ThemeProvider { Knob { value: 0.5 } } }
        }
        let mut v = VirtualDom::new(app);
        v.rebuild_in_place();
        let html = dioxus_ssr::render(&v);
        assert!(
            html.contains("dodai-knob") && html.contains("aria-valuenow"),
            "{html}"
        );
    }
}
