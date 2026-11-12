//! The orientation gizmo of a 3D view as inline SVG: stems and balls of the world axes for an
//! orbit camera and a home button. Click a ball to look along its axis, home to go back, drag to
//! orbit. Geometry and hit testing come from `dodai_core::orientation`.

use dioxus::prelude::*;
use dodai_core::orientation::{hit, orbit, place, AxisId, Basis, DragTracker, GizmoLayout, Hit};
use dodai_core::Theme;

pub use dodai_topcoat::html::gizmo_svg;

use crate::ThemeHandle;

/// The camera a gizmo starts at and goes home to: yaw and pitch in radians.
pub const GIZMO_HOME: (f32, f32) = (0.6, 0.45);

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
