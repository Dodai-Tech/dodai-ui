//! The orientation gizmo of a 3D view as inline SVG: stems and balls of the world axes for an
//! orbit camera and a home button. Click a ball to look along its axis, home to go back, drag to
//! orbit. Geometry and hit testing come from `dodai_core::orientation`.

use dioxus::prelude::*;
use dodai_core::orientation::{
    hit, orbit, place, AxisId, AxisTone, Basis, DragTracker, GizmoLayout, Hit,
};
use dodai_core::{Palette, Theme};

use crate::ThemeHandle;

/// The camera a gizmo starts at and goes home to: yaw and pitch in radians.
pub const GIZMO_HOME: (f32, f32) = (0.6, 0.45);

/// Colour of an axis family: X danger, Y success, Z primary.
fn tone(p: &Palette, t: AxisTone) -> dodai_core::Color {
    match t {
        AxisTone::X => p.danger,
        AxisTone::Y => p.success,
        AxisTone::Z => p.primary,
    }
}

/// The SVG of a gizmo `size` px across for an orbit camera at `yaw` / `pitch`.
pub fn gizmo_svg(theme: &Theme, size: f32, yaw: f32, pitch: f32) -> String {
    let p = &theme.palette;
    let layout = GizmoLayout::new(size);
    let placed = place(&Basis::from_orbit(yaw, pitch), &layout, &AxisId::DEFAULT);
    let [cx, cy] = layout.centre();
    let bw = theme.shape.border_width.max(1.0);
    let mut s = format!(
        "<circle cx=\"{cx}\" cy=\"{cy}\" r=\"{}\" fill=\"{}\" fill-opacity=\"0.72\" stroke=\"{}\" stroke-width=\"{bw}\"/>",
        size / 2.0 - bw / 2.0,
        p.surface.to_css(),
        p.border_strong.to_css(),
    );
    for a in &placed {
        let op = if a.axis.positive() { 1.0 } else { 0.55 };
        s.push_str(&format!(
            "<line x1=\"{cx}\" y1=\"{cy}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"{}\" stroke-opacity=\"{op}\" stroke-width=\"{}\" stroke-linecap=\"round\"/>",
            a.centre[0],
            a.centre[1],
            tone(p, a.axis.tone()).to_css(),
            layout.stem
        ));
    }
    for a in &placed {
        let c = tone(p, a.axis.tone());
        let (x, y) = (a.centre[0], a.centre[1]);
        if a.axis.positive() {
            s.push_str(&format!(
                "<circle cx=\"{x:.2}\" cy=\"{y:.2}\" r=\"{:.2}\" fill=\"{}\"/><text x=\"{x:.2}\" y=\"{y:.2}\" fill=\"{}\" font-size=\"{:.1}\" font-weight=\"700\" text-anchor=\"middle\" dominant-baseline=\"central\">{}</text>",
                a.radius,
                c.to_css(),
                theme.on(c).to_css(),
                a.radius * 2.0 * 0.55,
                a.axis.letter()
            ));
        } else {
            let w = (a.radius * 2.0 * 0.17).max(2.0);
            s.push_str(&format!(
                "<circle cx=\"{x:.2}\" cy=\"{y:.2}\" r=\"{:.2}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{w:.2}\"/>",
                a.radius - w / 2.0,
                c.to_css()
            ));
        }
    }
    let [hx, hy] = layout.home_centre();
    let hr = layout.home / 2.0;
    let k = hr * 0.55;
    s.push_str(&format!(
        "<circle cx=\"{hx:.2}\" cy=\"{hy:.2}\" r=\"{hr:.2}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"{bw}\"/><path d=\"M{:.2} {hy:.2} L{hx:.2} {:.2} L{:.2} {hy:.2} M{:.2} {:.2} V{:.2} H{:.2} V{:.2}\" fill=\"none\" stroke=\"{}\" stroke-width=\"1.5\"/>",
        p.surface_raised.to_css(),
        p.border.to_css(),
        hx - k,
        hy - k,
        hx + k,
        hx - k * 0.7,
        hy - k * 0.2,
        hy + k * 0.8,
        hx + k * 0.7,
        hy - k * 0.2,
        p.text.to_css(),
    ));
    s
}

/// The camera after a click at `at` (px from the top left): a ball's view, home, or `None`.
pub fn gizmo_click(size: f32, yaw: f32, pitch: f32, at: [f32; 2]) -> Option<(f32, f32)> {
    let layout = GizmoLayout::new(size);
    let placed = place(&Basis::from_orbit(yaw, pitch), &layout, &AxisId::DEFAULT);
    match hit(&layout, &placed, at) {
        Hit::Axis(a) => {
            let (y, p) = a.view_angles();
            Some((y.unwrap_or(yaw), p))
        }
        Hit::Home => Some(GIZMO_HOME),
        _ => None,
    }
}

/// Gizmo properties.
#[derive(Props, Clone, PartialEq)]
pub struct AxisGizmoProps {
    /// Camera yaw, radians.
    pub yaw: f32,
    /// Camera pitch, radians.
    pub pitch: f32,
    /// Side px.
    #[props(default = 130.0)]
    pub size: f32,
    /// Theme to draw with (default: the provider's).
    #[props(default)]
    pub theme: Option<Theme>,
    /// Called with the new camera `(yaw, pitch)`.
    #[props(default)]
    pub onchange: EventHandler<(f32, f32)>,
}

/// Orientation gizmo for a 3D view (Z up). Controlled: pass the camera.
#[component]
pub fn AxisGizmo(props: AxisGizmoProps) -> Element {
    let Some(theme) = props
        .theme
        .clone()
        .or_else(|| try_use_context::<ThemeHandle>().map(|h| h.theme()))
    else {
        return rsx! {};
    };
    let mut press = use_signal(|| None::<(f32, f32)>);
    let mut drag = use_signal(DragTracker::default);
    let (yaw, pitch, size, h) = (props.yaw, props.pitch, props.size, props.onchange);
    let svg = gizmo_svg(&theme, size, yaw, pitch);
    rsx! {
        svg { class: "dodai-gizmo", width: "{size}", height: "{size}", view_box: "0 0 {size} {size}",
            style: "cursor:pointer;touch-action:none;user-select:none",
            onmousedown: move |e| {
                let q = e.element_coordinates();
                press.set(Some((q.x as f32, q.y as f32)));
                drag.write().press();
            },
            onmousemove: move |e| if let Some((x, y)) = press() {
                let q = e.element_coordinates();
                let (dx, dy) = (q.x as f32 - x, q.y as f32 - y);
                drag.write().add(dx, dy);
                press.set(Some((q.x as f32, q.y as f32)));
                if drag().is_drag() {
                    h.call(orbit(yaw, pitch, dx, dy));
                }
            },
            onmouseup: move |e| {
                if press.take().is_some() && !drag().is_drag() {
                    let q = e.element_coordinates();
                    if let Some(cam) = gizmo_click(size, yaw, pitch, [q.x as f32, q.y as f32]) {
                        h.call(cam);
                    }
                }
            },
            onmouseleave: move |_| press.set(None),
            dangerous_inner_html: "{svg}"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dodai_core::themes;

    #[test]
    fn the_svg_has_the_disc_stems_balls_and_home() {
        let t = themes::all().remove(0);
        let s = gizmo_svg(&t, 130.0, GIZMO_HOME.0, GIZMO_HOME.1);
        assert_eq!(s.matches("<line").count(), AxisId::DEFAULT.len());
        assert!(s.contains(">X<") && s.contains(">Z<"));
        assert!(s.contains("<path"));
    }

    #[test]
    fn clicking_home_goes_home_and_outside_does_nothing() {
        let l = GizmoLayout::new(130.0);
        assert_eq!(
            gizmo_click(130.0, 0.0, 0.0, l.home_centre()),
            Some(GIZMO_HOME)
        );
        assert_eq!(gizmo_click(130.0, 0.0, 0.0, [0.0, 129.0]), None);
    }
}
