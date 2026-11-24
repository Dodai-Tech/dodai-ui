//! Orientation gizmo: the corner widget of a 3D view that shows where the world axes point and
//! lets the user jump to a standard view or drag to orbit.
//!
//! Everything here is pure geometry in a **Z-up** world (X east, Y north, Z up). A toolkit
//! feeds in the camera [`Basis`], paints the [`Placed`] balls and stems back to front, and
//! turns pointer positions into a [`Hit`]. Dragging is a click until it moves more than
//! [`DRAG_SLOP`] pixels ([`DragTracker`]), so the same press can orbit or select.

use std::f32::consts::FRAC_PI_2;

/// Pixels a press may move and still count as a click.
pub const DRAG_SLOP: f32 = 4.0;
/// Radians of orbit per dragged pixel.
pub const DRAG_RADIANS_PER_PX: f32 = 0.005;
/// Camera elevation of the side views, radians (a slight look down).
pub const SIDE_PITCH: f32 = 0.12;
/// Steepest orbit elevation, radians (just short of straight up or down).
pub const MAX_PITCH: f32 = 1.5;

/// A world axis direction the gizmo can show.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AxisId {
    /// +X, east.
    PosX,
    /// +Y, north.
    PosY,
    /// +Z, up.
    PosZ,
    /// -X, west.
    NegX,
    /// -Y, south.
    NegY,
    /// -Z, down.
    NegZ,
}

/// Which colour family an axis uses: X, Y and Z each have one, shared by both signs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AxisTone {
    /// The X axis.
    X,
    /// The Y axis.
    Y,
    /// The Z axis.
    Z,
}

impl AxisId {
    /// All six axes.
    pub const ALL: [AxisId; 6] = [
        AxisId::PosX,
        AxisId::PosY,
        AxisId::PosZ,
        AxisId::NegX,
        AxisId::NegY,
        AxisId::NegZ,
    ];
    /// The usual set: three positive axes and the horizontal negatives (no view from below).
    pub const DEFAULT: [AxisId; 5] = [
        AxisId::PosX,
        AxisId::PosY,
        AxisId::PosZ,
        AxisId::NegX,
        AxisId::NegY,
    ];

    /// Unit direction in the world.
    #[must_use]
    pub fn dir(self) -> [f32; 3] {
        match self {
            AxisId::PosX => [1.0, 0.0, 0.0],
            AxisId::PosY => [0.0, 1.0, 0.0],
            AxisId::PosZ => [0.0, 0.0, 1.0],
            AxisId::NegX => [-1.0, 0.0, 0.0],
            AxisId::NegY => [0.0, -1.0, 0.0],
            AxisId::NegZ => [0.0, 0.0, -1.0],
        }
    }

    /// `true` for +X, +Y and +Z: those get a filled ball with a letter, the others a ring.
    #[must_use]
    pub fn positive(self) -> bool {
        matches!(self, AxisId::PosX | AxisId::PosY | AxisId::PosZ)
    }

    /// `"X"`, `"Y"` or `"Z"`.
    #[must_use]
    pub fn letter(self) -> &'static str {
        match self.tone() {
            AxisTone::X => "X",
            AxisTone::Y => "Y",
            AxisTone::Z => "Z",
        }
    }

    /// The colour family.
    #[must_use]
    pub fn tone(self) -> AxisTone {
        match self {
            AxisId::PosX | AxisId::NegX => AxisTone::X,
            AxisId::PosY | AxisId::NegY => AxisTone::Y,
            AxisId::PosZ | AxisId::NegZ => AxisTone::Z,
        }
    }

    /// Orbit angles that put the camera on this axis: `(yaw, pitch)` as in
    /// [`Basis::from_orbit`]. The yaw is `None` for the vertical axes (keep the current one).
    #[must_use]
    pub fn view_angles(self) -> (Option<f32>, f32) {
        match self {
            AxisId::NegY => (Some(0.0), SIDE_PITCH),
            AxisId::PosX => (Some(FRAC_PI_2), SIDE_PITCH),
            AxisId::PosY => (Some(2.0 * FRAC_PI_2), SIDE_PITCH),
            AxisId::NegX => (Some(-FRAC_PI_2), SIDE_PITCH),
            AxisId::PosZ => (None, MAX_PITCH),
            AxisId::NegZ => (None, -MAX_PITCH),
        }
    }
}

/// The camera's axes in world coordinates: what the user sees as right, up and towards them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Basis {
    /// Screen-right in the world.
    pub right: [f32; 3],
    /// Screen-up in the world.
    pub up: [f32; 3],
    /// From the scene towards the viewer.
    pub back: [f32; 3],
}

impl Basis {
    /// Basis of a camera orbiting a target in a Z-up world. Yaw 0 looks north from the south
    /// side and a positive yaw swings the camera east; pitch is the elevation (positive looks
    /// down on the scene). Pitch is clamped to ±[`MAX_PITCH`].
    #[must_use]
    pub fn from_orbit(yaw: f32, pitch: f32) -> Self {
        let p = pitch.clamp(-MAX_PITCH, MAX_PITCH);
        let back = [yaw.sin() * p.cos(), -yaw.cos() * p.cos(), p.sin()];
        // right = forward x world-up, with forward = -back.
        let right = normalize([-back[1], back[0], 0.0]);
        let up = cross(back, right);
        Self { right, up, back }
    }
}

/// New `(yaw, pitch)` after dragging `dx`, `dy` pixels (right and down on screen): the scene
/// follows the pointer.
#[must_use]
pub fn orbit(yaw: f32, pitch: f32, dx: f32, dy: f32) -> (f32, f32) {
    (
        yaw - dx * DRAG_RADIANS_PER_PX,
        (pitch + dy * DRAG_RADIANS_PER_PX).clamp(-MAX_PITCH, MAX_PITCH),
    )
}

/// Sizes of the gizmo, all derived from its side length in pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GizmoLayout {
    /// Side of the square the round gizmo fits in.
    pub size: f32,
    /// Distance from the centre to an axis tip.
    pub reach: f32,
    /// Diameter of a positive (filled) ball.
    pub ball: f32,
    /// Diameter of a negative (hollow) ring.
    pub ring: f32,
    /// Thickness of a stem.
    pub stem: f32,
    /// Diameter of the home button.
    pub home: f32,
    /// Diameter of the zoom (plus and minus) buttons.
    pub zoom: f32,
}

impl GizmoLayout {
    /// Layout for a gizmo `size` pixels wide.
    #[must_use]
    pub fn new(size: f32) -> Self {
        Self {
            size,
            reach: size * 0.29,
            ball: size * 0.2,
            ring: size * 0.14,
            stem: (size * 0.03).max(3.0),
            home: size * 0.23,
            zoom: size * 0.2,
        }
    }

    /// Centre of the gizmo, from the top-left of its square.
    #[must_use]
    pub fn centre(&self) -> [f32; 2] {
        [self.size / 2.0, self.size / 2.0]
    }

    /// Centre of the home button: on the circle line, top left.
    #[must_use]
    pub fn home_centre(&self) -> [f32; 2] {
        let r = self.size / 2.0 * std::f32::consts::FRAC_1_SQRT_2;
        [self.size / 2.0 - r, self.size / 2.0 - r]
    }

    /// Centre of the zoom-out (minus) button: on the circle line, bottom left.
    #[must_use]
    pub fn zoom_out_centre(&self) -> [f32; 2] {
        let r = self.size / 2.0 * std::f32::consts::FRAC_1_SQRT_2;
        [self.size / 2.0 - r, self.size / 2.0 + r]
    }

    /// Centre of the zoom-in (plus) button: on the circle line, bottom right.
    #[must_use]
    pub fn zoom_in_centre(&self) -> [f32; 2] {
        let r = self.size / 2.0 * std::f32::consts::FRAC_1_SQRT_2;
        [self.size / 2.0 + r, self.size / 2.0 + r]
    }
}

/// An axis laid out for painting.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placed {
    /// Which axis.
    pub axis: AxisId,
    /// Centre of its ball, from the top-left of the gizmo square (y down).
    pub centre: [f32; 2],
    /// Towards the viewer is positive; paint in ascending order.
    pub depth: f32,
    /// Radius of its ball.
    pub radius: f32,
}

/// Lays out `axes` for a camera `basis`, sorted back to front (paint in this order, each stem
/// just before its ball). The stem of an axis runs from [`GizmoLayout::centre`] to its centre.
#[must_use]
pub fn place(basis: &Basis, layout: &GizmoLayout, axes: &[AxisId]) -> Vec<Placed> {
    let c = layout.centre();
    let mut out: Vec<Placed> = axes
        .iter()
        .map(|&axis| {
            let d = axis.dir();
            let (x, y, z) = (dot(d, basis.right), dot(d, basis.up), dot(d, basis.back));
            let diameter = if axis.positive() {
                layout.ball
            } else {
                layout.ring
            };
            Placed {
                axis,
                centre: [c[0] + x * layout.reach, c[1] - y * layout.reach],
                depth: z,
                radius: diameter / 2.0,
            }
        })
        .collect();
    out.sort_by(|a, b| a.depth.total_cmp(&b.depth));
    out
}

/// What a pointer position is over.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Hit {
    /// An axis ball: select that view.
    Axis(AxisId),
    /// The home button.
    Home,
    /// Inside the circle, on nothing else: dragging here orbits.
    Body,
    /// Outside the gizmo.
    Outside,
}

/// Hit test of `point` (from the top-left of the square). Home wins over the circle, the
/// nearest ball over the ones behind it.
#[must_use]
pub fn hit(layout: &GizmoLayout, placed: &[Placed], point: [f32; 2]) -> Hit {
    let home = layout.home_centre();
    if distance(point, home) <= layout.home / 2.0 {
        return Hit::Home;
    }
    for p in placed.iter().rev() {
        if distance(point, p.centre) <= p.radius {
            return Hit::Axis(p.axis);
        }
    }
    if distance(point, layout.centre()) <= layout.size / 2.0 {
        Hit::Body
    } else {
        Hit::Outside
    }
}

/// Tells a click from a drag: a press that travels further than [`DRAG_SLOP`] is a drag.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DragTracker {
    moved: f32,
}

impl DragTracker {
    /// Starts counting at a new press.
    pub fn press(&mut self) {
        self.moved = 0.0;
    }
    /// Adds a pointer movement while pressed.
    pub fn add(&mut self, dx: f32, dy: f32) {
        self.moved += dx.hypot(dy);
    }
    /// `true` once the press travelled past the slop: do not treat the release as a click.
    #[must_use]
    pub fn is_drag(&self) -> bool {
        self.moved > DRAG_SLOP
    }
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let n = dot(v, v).sqrt();
    if n < 1e-6 {
        [1.0, 0.0, 0.0]
    } else {
        [v[0] / n, v[1] / n, v[2] / n]
    }
}

fn distance(a: [f32; 2], b: [f32; 2]) -> f32 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

/// The camera the gizmo's home button returns to: `(yaw, pitch)` in radians.
pub const GIZMO_HOME: (f32, f32) = (0.6, 0.45);

/// What a press at `at` (gizmo-local px) does to an orbit camera: the new `(yaw, pitch)` for a
/// press on an axis ball (look along it) or on home, none elsewhere.
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

#[cfg(test)]
mod tests {
    use super::*;

    fn near(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    #[test]
    fn corner_buttons_sit_on_the_circle_line() {
        let l = GizmoLayout::new(130.0);
        let c = l.centre();
        for p in [l.home_centre(), l.zoom_out_centre(), l.zoom_in_centre()] {
            let d = (p[0] - c[0]).hypot(p[1] - c[1]);
            assert!(near(d, 65.0), "on the circle: {d}");
        }
        // home top left, minus bottom left, plus bottom right
        assert!(l.home_centre()[1] < c[1] && l.home_centre()[0] < c[0]);
        assert!(l.zoom_out_centre()[1] > c[1] && l.zoom_out_centre()[0] < c[0]);
        assert!(l.zoom_in_centre()[1] > c[1] && l.zoom_in_centre()[0] > c[0]);
        assert!(l.zoom > 0.0);
    }

    #[test]
    fn basis_is_orthonormal_and_south_view_has_x_right() {
        let b = Basis::from_orbit(0.0, 0.3);
        assert!(near(dot(b.right, b.up), 0.0) && near(dot(b.up, b.back), 0.0));
        assert!(near(dot(b.right, b.right), 1.0) && near(dot(b.up, b.up), 1.0));
        assert!(near(b.right[0], 1.0), "east is to the right from the south");
        assert!(b.up[2] > 0.9, "up stays up");
    }

    #[test]
    fn every_axis_view_faces_the_viewer() {
        for axis in AxisId::ALL {
            let (yaw, pitch) = axis.view_angles();
            let b = Basis::from_orbit(yaw.unwrap_or(0.4), pitch);
            assert!(dot(axis.dir(), b.back) > 0.9, "{axis:?}");
        }
    }

    #[test]
    fn nearest_axis_is_painted_last() {
        let layout = GizmoLayout::new(130.0);
        let b = Basis::from_orbit(0.0, 0.0);
        let placed = place(&b, &layout, &AxisId::DEFAULT);
        assert_eq!(placed.last().map(|p| p.axis), Some(AxisId::NegY));
        assert!(placed.windows(2).all(|w| w[0].depth <= w[1].depth));
    }

    #[test]
    fn hit_prefers_home_then_front_ball_then_body() {
        let layout = GizmoLayout::new(130.0);
        let b = Basis::from_orbit(0.0, 0.5);
        let placed = place(&b, &layout, &AxisId::DEFAULT);
        assert_eq!(hit(&layout, &placed, layout.home_centre()), Hit::Home);
        let z = placed.iter().find(|p| p.axis == AxisId::PosZ).expect("z");
        assert_eq!(hit(&layout, &placed, z.centre), Hit::Axis(AxisId::PosZ));
        assert_eq!(hit(&layout, &placed, [65.0, 120.0]), Hit::Body);
        assert_eq!(hit(&layout, &placed, [-5.0, -5.0]), Hit::Outside);
    }

    #[test]
    fn drag_follows_the_pointer_and_clamps_pitch() {
        let (yaw, pitch) = orbit(0.0, 0.0, 100.0, 0.0);
        assert!(
            yaw < 0.0 && near(pitch, 0.0),
            "drag right turns the scene right"
        );
        assert!(near(orbit(0.0, 0.0, 0.0, 1e6).1, MAX_PITCH));
        assert!(near(orbit(0.0, 0.0, 0.0, -1e6).1, -MAX_PITCH));
    }

    #[test]
    fn small_movement_is_still_a_click() {
        let mut d = DragTracker::default();
        d.press();
        d.add(1.0, 1.0);
        assert!(!d.is_drag());
        d.add(5.0, 0.0);
        assert!(d.is_drag());
        d.press();
        assert!(!d.is_drag());
    }
}
