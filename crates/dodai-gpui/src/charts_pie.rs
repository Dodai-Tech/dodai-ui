//! Pie and donut chart.

use crate::charts::*;
use crate::text::first_family;
use crate::{hsla, theme_of};
use dodai_core::Color;
use gpui::{
    canvas, div, point, px, App, Bounds, Hsla, InteractiveElement, IntoElement, ParentElement,
    Pixels, SharedString, StatefulInteractiveElement, Styled, Window,
};
use std::rc::Rc;

// ───────────────────────────── PieChart / donut ─────────────────────────────

type SelectFn = Rc<dyn Fn(Option<usize>, &mut Window, &mut App)>;

/// How far (px) a hovered pie slice pops out.
const PIE_POP: f32 = 6.0;

#[derive(Default)]
struct PieState {
    bounds: Bounds<Pixels>,
    hover: Option<usize>,
    sel: Option<usize>,
    pops: Vec<crate::motion::Tween>,
}

/// Slice index under a pointer `(dx, dy)` from the pie centre (y down), or
/// `None` outside the ring / disc. `fr` are `(start, end)` turn fractions,
/// 0 at the top, clockwise. Pure.
pub fn pie_hit(dx: f32, dy: f32, r: f32, hole: f32, fr: &[(f32, f32)]) -> Option<usize> {
    let d = (dx * dx + dy * dy).sqrt();
    if d > r + PIE_POP || d < r * hole {
        return None;
    }
    let turn = (dx.atan2(-dy) / std::f32::consts::TAU).rem_euclid(1.0);
    // Fractions may run past 1 (a partial donut starting at 0.75).
    fr.iter()
        .position(|(a, z)| (turn >= *a && turn < *z) || (turn + 1.0 >= *a && turn + 1.0 < *z))
}

/// How a [`PieChart`] draws its slices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PieStyle {
    /// Solid wedges / rings (default).
    #[default]
    Solid,
    /// Ring built from separate cells, like a segmented readout.
    Segmented,
    /// Thin glowing band between an outer tick scale and a dashed inner ring.
    Hud,
    /// Radius follows the value (a rose / polar-area chart) over guide rings.
    Rose,
}

/// Pie or donut chart; `center` is shown in the donut hole.
#[derive(IntoElement)]
pub struct PieChart {
    style: PieStyle,
    slices: Vec<(SharedString, f32)>,
    size: f32,
    hole: f32,
    center: Option<SharedString>,
    legend: bool,
    selected: Option<Option<usize>>,
    select_color: Option<Color>,
    on_select: Option<SelectFn>,
    gap: f32,
    explode: f32,
    colors: Option<Vec<Color>>,
    start: f32,
    sweep: f32,
}

impl PieChart {
    /// Create a new [`PieChart`].
    pub fn new<S: Into<SharedString>>(slices: impl IntoIterator<Item = (S, f32)>) -> Self {
        Self {
            style: PieStyle::Solid,
            slices: slices.into_iter().map(|(n, v)| (n.into(), v)).collect(),
            size: 160.0,
            hole: 0.0,
            center: None,
            legend: true,
            selected: None,
            select_color: None,
            on_select: None,
            gap: 2.0,
            explode: 0.0,
            colors: None,
            start: 0.0,
            sweep: 1.0,
        }
    }
    /// Look of the slices (default [`PieStyle::Solid`]). `Segmented` and `Hud`
    /// are rings: they use at least a 0.55 / 0.76 hole.
    pub fn style(mut self, s: PieStyle) -> Self {
        self.style = s;
        self
    }
    /// Straight gap between slices in px (default 2, `0` = flush).
    pub fn gap(mut self, g: f32) -> Self {
        self.gap = g.max(0.0);
        self
    }
    /// Push every slice outward by `px` (an exploded pie), up to 10.
    pub fn explode(mut self, px: f32) -> Self {
        self.explode = px.clamp(0.0, 10.0);
        self
    }
    /// Slice colours (cycled) instead of the theme's series colours, e.g. a
    /// monochrome ramp.
    pub fn colors(mut self, c: impl IntoIterator<Item = Color>) -> Self {
        self.colors = Some(c.into_iter().collect());
        self
    }
    /// Draw only `sweep` of a full turn (`0.5` = half donut), beginning at
    /// `start` turns clockwise from the top (`0.75` = from the left).
    pub fn arc(mut self, start: f32, sweep: f32) -> Self {
        self.start = start;
        self.sweep = sweep.clamp(0.05, 1.0);
        self
    }
    /// Controlled selection (`None` = nothing selected). Without this the
    /// chart keeps its own selection, toggled by clicking a slice.
    pub fn selected(mut self, i: Option<usize>) -> Self {
        self.selected = Some(i);
        self
    }
    /// Outline colour of the selected slice (default: the theme accent).
    pub fn selected_color(mut self, c: Color) -> Self {
        self.select_color = Some(c);
        self
    }
    crate::callback!(
        /// Called with the new selection (`None` when deselected) on click.
        on_select, Option<usize>
    );
    /// Size in px.
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
    /// Donut hole as a fraction of the radius (`0` = pie).
    pub fn donut(mut self, hole: f32) -> Self {
        self.hole = hole.clamp(0.0, 0.9);
        self
    }
    /// Text shown in the centre.
    pub fn center(mut self, c: impl Into<SharedString>) -> Self {
        self.center = Some(c.into());
        self
    }
    /// Show or hide the legend.
    pub fn legend(mut self, l: bool) -> Self {
        self.legend = l;
        self
    }
}

/// Where one slice sits: centre, radii, and its edges before and after the constant-width gap.
#[derive(Clone, Copy)]
struct SliceGeo {
    c: f32,
    r: f32,
    hole: f32,
    /// Half the gap between slices, px.
    half: f32,
    edges: (f32, f32),
    trimmed: (f32, f32),
}

/// How one slice is drawn.
#[derive(Clone, Copy)]
struct SliceLook {
    col: Hsla,
    ring: Hsla,
    selected: bool,
    glow: Option<f32>,
    hud: bool,
    soft: bool,
    /// Ink outline of the design's chart style: colour and width.
    outline: Option<(Hsla, f32)>,
}

/// Outer tick scale and dashed inner reference ring of a HUD donut; only the part of the turn
/// the arc covers is decorated.
fn hud_scale(
    w: &mut Window,
    origin: gpui::Point<Pixels>,
    c: f32,
    r: f32,
    hole: f32,
    (start, sweep): (f32, f32),
    (guide_col, scale_col): (Hsla, Hsla),
) {
    // Outer tick scale + dashed inner reference ring.
    let (mut minor, mut major) = (Vec::new(), Vec::new());
    // Only decorate the part of the turn the arc covers.
    let within = |u: f32| ((u - start).rem_euclid(1.0)) <= sweep + 0.001;
    for k in 0..72 {
        let u = k as f32 / 72.0;
        if !within(u) {
            continue;
        }
        let long = k % 6 == 0;
        let seg = vec![
            polar(c, c, r + 4.0, u),
            polar(c, c, r + if long { 11.0 } else { 7.0 }, u),
        ];
        if long {
            major.push(seg);
        } else {
            minor.push(seg);
        }
    }
    stroke_batch(w, origin, &minor, 1.0, guide_col, false);
    stroke_batch(w, origin, &major, 1.5, scale_col, false);
    let dashes: Vec<Vec<(f32, f32)>> = (0..48)
        .filter(|k| within(*k as f32 / 48.0))
        .map(|k| {
            let u = k as f32 / 48.0;
            turn_arc(c, c, r * hole - 6.0, u, u + 0.01)
        })
        .collect();
    stroke_batch(w, origin, &dashes, 1.0, scale_col, false);
}

/// Faint reference rings behind a rose chart.
fn rose_guides(w: &mut Window, origin: gpui::Point<Pixels>, c: f32, r: f32, col: Hsla) {
    for f in [0.38, 0.69, 1.0] {
        stroke(w, origin, &turn_arc(c, c, r * f, 0.0, 1.0), 1.0, col, true);
    }
}

/// A segmented-ring slice: small filled cells with gaps between them.
fn segmented_slice(w: &mut Window, origin: gpui::Point<Pixels>, geo: &SliceGeo, look: &SliceLook) {
    let SliceGeo {
        c,
        r,
        hole,
        trimmed: (ao, zo),
        ..
    } = *geo;
    let SliceLook {
        col,
        ring,
        selected,
        soft,
        ..
    } = *look;
    let r_in = r * hole;
    let per_turn = if soft { 16.0 } else { 36.0 };
    let n = (((zo - ao) * per_turn).ceil() as usize).max(1);
    let g = if soft { 0.012 } else { 0.004 };
    let mut cells = Vec::new();
    for k in 0..n {
        let c0 = ao + (zo - ao) * k as f32 / n as f32 + g / 2.0;
        let c1 = ao + (zo - ao) * (k + 1) as f32 / n as f32 - g / 2.0;
        if c1 <= c0 {
            continue;
        }
        let mut p = turn_arc(c, c, r, c0, c1);
        let mut inner = turn_arc(c, c, r_in, c0, c1);
        inner.reverse();
        p.extend(inner);
        cells.push(p);
    }
    fill_batch(w, origin, &cells, col);
    if selected {
        stroke(
            w,
            origin,
            &turn_arc(c, c, r + 2.0, ao, zo),
            2.0,
            ring,
            false,
        );
    }
}

/// A pie slice (no hole): convex wedges meeting at the gap-offset apex.
fn wedge_slice(w: &mut Window, origin: gpui::Point<Pixels>, geo: &SliceGeo, look: &SliceLook) {
    let SliceGeo {
        c,
        r,
        half,
        edges: (a, z),
        trimmed: (ao, zo),
        ..
    } = *geo;
    let SliceLook {
        col,
        ring,
        selected,
        outline,
        ..
    } = *look;
    let theta = (z - a) * std::f32::consts::TAU;
    let d = if half > 0.0 {
        half / (theta / 2.0).sin().max(0.05)
    } else {
        0.0
    };
    let (ax, ay) = polar(c, c, d, (a + z) / 2.0);
    // Convex wedges (<= half a turn) tessellate without
    // holes; fewer pieces means fewer AA seams.
    let n = (((zo - ao) * 2.0).ceil() as usize).max(1);
    for k in 0..n {
        let w0 = ao + (zo - ao) * k as f32 / n as f32;
        let w1 = ao + (zo - ao) * (k + 1) as f32 / n as f32;
        let mut p = turn_arc(c, c, r, w0, if k + 1 == n { w1 } else { w1 + 0.0015 });
        p.push((ax, ay));
        fill(w, origin, &p, col);
    }
    if let Some((ink, width)) = outline {
        let mut o = turn_arc(c, c, r, ao, zo);
        o.push((ax, ay));
        stroke(w, origin, &o, width, ink, true);
    }
    if selected {
        let mut o = turn_arc(c, c, r, ao, zo);
        o.push((ax, ay));
        stroke(w, origin, &o, 1.5, ring, true);
    }
}

/// A donut slice: annular sectors as small filled quads (thick strokes leave mitre seams).
fn annular_slice(w: &mut Window, origin: gpui::Point<Pixels>, geo: &SliceGeo, look: &SliceLook) {
    let SliceGeo {
        c,
        r,
        hole,
        half,
        edges: (a, z),
        trimmed: (ao, zo),
    } = *geo;
    let SliceLook {
        col,
        ring,
        selected,
        glow,
        hud,
        outline,
        ..
    } = *look;
    let delta = |rad: f32| (half / rad).clamp(0.0, 1.0).asin() / std::f32::consts::TAU;
    // Annular sectors as small filled quads: thick strokes
    // leave mitre artefacts and seams, fills do not.
    let r_in = r * hole;
    let (ai, zi) = (a + delta(r_in), z - delta(r_in));
    let n = (((zo - ao) * 24.0).ceil() as usize).max(1);
    for k in 0..n {
        let f0 = k as f32 / n as f32;
        let f1 = (k + 1) as f32 / n as f32;
        let e = if k + 1 == n { 0.0 } else { 0.001 };
        let mut p = turn_arc(c, c, r, ao + (zo - ao) * f0, ao + (zo - ao) * f1 + e);
        let mut inner = turn_arc(c, c, r_in, ai + (zi - ai) * f0, ai + (zi - ai) * f1 + e);
        inner.reverse();
        p.extend(inner);
        fill(w, origin, &p, col);
    }
    if hud {
        glow_stroke(w, origin, &turn_arc(c, c, r + 2.0, ao, zo), 1.5, col, glow);
    }
    if let Some((ink, width)) = outline {
        let mut o = turn_arc(c, c, r, ao, zo);
        let mut inner = turn_arc(c, c, r_in, ai, zi);
        inner.reverse();
        o.extend(inner);
        stroke(w, origin, &o, width, ink, true);
    }
    if selected {
        let mut o = turn_arc(c, c, r, ao, zo);
        let mut inner = turn_arc(c, c, r_in, ai, zi);
        inner.reverse();
        o.extend(inner);
        stroke(w, origin, &o, 1.5, ring, true);
    }
}

/// The selection after a click on `hit`: clicking the selected slice (or empty space) clears it.
fn next_selection(cur: Option<usize>, hit: Option<usize>) -> Option<usize> {
    hit.filter(|i| cur != Some(*i))
}

/// The value shown in the middle of the ring.
fn center_overlay(
    t: &dodai_core::Theme,
    txt: SharedString,
    size: f32,
    top_half: bool,
) -> gpui::Div {
    let overlay = div().absolute().left_0().w(px(size));
    // Top half ring: centre the value on the baseline, not the full circle.
    let overlay = if top_half {
        overlay.bottom_0().h(px(size * 0.3))
    } else {
        overlay.top_0().h(px(size))
    };
    overlay
        .flex()
        .items_center()
        .justify_center()
        .font_family(first_family(t.typography.display))
        .text_size(px(t.typography.size_xl))
        .text_color(hsla(t.palette.text))
        .child(txt)
}

/// The legend: a swatch, the name and its share per slice.
fn legend(
    t: &dodai_core::Theme,
    slices: &[(SharedString, f32)],
    cols: &[Hsla],
    total: f32,
) -> gpui::Div {
    div()
        .flex()
        .flex_col()
        .gap(px(t.spacing.xs))
        .children(slices.iter().enumerate().map(|(i, (n, v))| {
            let pct = if total > 0.0 { v / total * 100.0 } else { 0.0 };
            label_style(t, div())
                .flex()
                .items_center()
                .gap(px(t.spacing.xs))
                .child(
                    div()
                        .w(px(t.sizes().legend_swatch))
                        .h(px(t.sizes().legend_swatch))
                        .bg(cols[i]),
                )
                .child(format!("{n} · {}%", trim(pct)))
        }))
}

/// Pointer handling for a pie: hover, click-to-select and leaving.
struct PieInput {
    st: gpui::Entity<PieState>,
    /// Slice edges in turns, before the grow-in animation.
    fr: Vec<(f32, f32)>,
    size: f32,
    reserve: f32,
    hole: f32,
    /// Whether the app owns the selection (`PieChart::selected`).
    controlled: bool,
    sel: Option<usize>,
    on_select: Option<SelectFn>,
}

impl PieInput {
    /// The slice under `pos`, if any.
    fn hit(&self, pos: gpui::Point<Pixels>, cx: &App) -> Option<usize> {
        let b = self.st.read(cx).bounds;
        let (ox, oy): (f32, f32) = (b.origin.x.into(), b.origin.y.into());
        let (mx, my): (f32, f32) = (pos.x.into(), pos.y.into());
        pie_hit(
            mx - ox - self.size / 2.0,
            my - oy - self.size / 2.0,
            self.size / 2.0 - 1.0 - self.reserve,
            self.hole,
            &self.fr,
        )
    }

    fn moved(&self, pos: gpui::Point<Pixels>, w: &mut Window, cx: &mut App) {
        let h = self.hit(pos, cx);
        if self.st.read(cx).hover != h {
            self.st.update(cx, |s, _| s.hover = h);
            w.refresh();
        }
    }

    fn clicked(&self, pos: gpui::Point<Pixels>, w: &mut Window, cx: &mut App) {
        let h = self.hit(pos, cx);
        let cur = if self.controlled {
            self.sel
        } else {
            self.st.read(cx).sel
        };
        let next = next_selection(cur, h);
        if !self.controlled {
            self.st.update(cx, |s, _| s.sel = next);
            w.refresh();
        }
        if let Some(f) = &self.on_select {
            f(next, w, cx);
        }
    }

    fn hover_changed(&self, over: bool, w: &mut Window, cx: &mut App) {
        if !over && self.st.read(cx).hover.is_some() {
            self.st.update(cx, |s, _| s.hover = None);
            w.refresh();
        }
    }
}

/// Everything the pie's canvas needs to paint one frame.
struct PiePaint {
    st: gpui::Entity<PieState>,
    fr: Vec<(f32, f32)>,
    cols: Vec<Hsla>,
    rfac: Vec<f32>,
    hover: Option<usize>,
    sel: Option<usize>,
    ring: Hsla,
    motion: crate::motion::Motion,
    gap: f32,
    glow: Option<f32>,
    hud: bool,
    soft: bool,
    style: PieStyle,
    hole: f32,
    size: f32,
    explode: f32,
    reserve: f32,
    start: f32,
    sweep: f32,
    guide_col: Hsla,
    scale_col: Hsla,
    outline: Option<(Hsla, f32)>,
}

impl PiePaint {
    fn paint(&self, b: Bounds<Pixels>, w: &mut Window, cx: &mut App) {
        let Self {
            st: st_paint,
            fr,
            cols,
            rfac,
            hover,
            sel,
            ring,
            motion,
            gap,
            glow,
            hud,
            soft,
            style,
            hole,
            size,
            explode,
            reserve,
            start,
            sweep,
            guide_col,
            scale_col,
            outline,
        } = self;
        let (
            hover,
            sel,
            ring,
            motion,
            gap,
            glow,
            hud,
            soft,
            style,
            hole,
            size,
            explode,
            reserve,
            start,
            sweep,
            guide_col,
            scale_col,
        ) = (
            *hover, *sel, *ring, *motion, *gap, *glow, *hud, *soft, *style, *hole, *size, *explode,
            *reserve, *start, *sweep, *guide_col, *scale_col,
        );
        st_paint.update(cx, |s, _| s.bounds = b);
        let r = size / 2.0 - 1.0 - reserve;
        let c = size / 2.0;
        if hud {
            hud_scale(
                w,
                b.origin,
                c,
                r,
                hole,
                (start, sweep),
                (guide_col, scale_col),
            );
        }
        if style == PieStyle::Rose {
            rose_guides(w, b.origin, c, r, guide_col);
        }
        for (i, ((a, z), col)) in fr.iter().zip(cols).enumerate() {
            let r = r * rfac[i];
            // Hovered slice pops out along its bisector.
            let target = explode
                + if hover == Some(i) {
                    PIE_POP
                } else if sel == Some(i) {
                    PIE_POP * 0.5
                } else {
                    0.0
                };
            let (pop, busy) = st_paint.update(cx, |s, _| {
                if s.pops.len() <= i {
                    s.pops.resize(i + 1, crate::motion::Tween::new(0.0, motion));
                }
                s.pops[i].to(target, motion, crate::motion::now());
                let now = crate::motion::now();
                (s.pops[i].value(now), !s.pops[i].is_done(now))
            });
            if busy {
                crate::motion::next_frame(w);
            }
            let (dx, dy) = polar(0.0, 0.0, pop, (a + z) / 2.0);
            let origin = point(b.origin.x + px(dx), b.origin.y + px(dy));
            // Constant-width straight gap: each edge is offset by
            // `gap / 2` from the true radial line, so the gap has
            // parallel sides instead of widening towards the rim.
            let half = gap / 2.0;
            let (a, z) = (*a, *z);
            let delta = |rad: f32| (half / rad).clamp(0.0, 1.0).asin() / std::f32::consts::TAU;
            let (ao, zo) = (a + delta(r), z - delta(r));
            if zo <= ao {
                continue;
            }
            let geo = SliceGeo {
                c,
                r,
                hole,
                half,
                edges: (a, z),
                trimmed: (ao, zo),
            };
            let look = SliceLook {
                col: *col,
                ring,
                selected: sel == Some(i),
                glow,
                hud,
                soft,
                outline: *outline,
            };
            match (style, hole <= 0.0) {
                (PieStyle::Segmented, _) => segmented_slice(w, origin, &geo, &look),
                (_, true) => wedge_slice(w, origin, &geo, &look),
                _ => annular_slice(w, origin, &geo, &look),
            }
        }
    }
}

impl PieChart {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let vals: Vec<f32> = self.slices.iter().map(|s| s.1).collect();
        let (start, sweep) = (self.start, self.sweep);
        let fr: Vec<(f32, f32)> = pie_fractions(&vals)
            .iter()
            .map(|(a, z)| (start + a * sweep, start + z * sweep))
            .collect();
        // Pies that differ in look must not share state: their pop targets
        // would fight and the tween would never settle (a redraw every frame).
        let key: SharedString = format!(
            "pie-{}-{:?}-{}-{}-{}-{}-{}",
            self.slices
                .iter()
                .map(|s| s.0.as_ref())
                .collect::<Vec<_>>()
                .join("|"),
            self.style,
            self.explode,
            self.hole,
            self.start,
            self.sweep,
            self.size,
        )
        .into();
        let st = window.use_keyed_state(key.clone(), cx, |_, _| PieState::default());
        let hover = st.read(cx).hover;
        let sel = self.selected.unwrap_or(st.read(cx).sel);
        let ring = hsla(self.select_color.unwrap_or(t.palette.primary));
        let motion = crate::motion::current(cx);
        let (st_paint, st_move) = (st.clone(), st.clone());
        let (controlled, on_select) = (self.selected.is_some(), self.on_select.clone());
        let fr_hit = fr.clone();
        let g = crate::motion::grow(window, cx, "PieChart");
        let fr: Vec<(f32, f32)> = fr
            .iter()
            .map(|(a, z)| (start + (a - start) * g, start + (z - start) * g))
            .collect();
        let slice_col = {
            let (t2, custom) = (t.clone(), self.colors.clone());
            move |i: usize| match &custom {
                Some(c) if !c.is_empty() => c[i % c.len()],
                _ => t2.palette.series_at(i),
            }
        };
        let cols: Vec<Hsla> = (0..vals.len()).map(|i| hsla(slice_col(i))).collect();
        let style = self.style;
        let hole = match style {
            PieStyle::Segmented => self.hole.max(0.55),
            PieStyle::Hud => self.hole.max(0.76),
            _ => self.hole,
        };
        let size = self.size;
        let explode = self.explode;
        // Rounded (Material-like) themes get the calm version: no tick scale
        // or dashed ring, fewer and wider-spaced segments.
        let soft = t.look().soft_elevation;
        let hud = style == PieStyle::Hud && !soft;
        let reserve = PIE_POP + explode + if hud { 12.0 } else { 0.0 };
        let vmax = vals
            .iter()
            .cloned()
            .fold(0.0_f32, f32::max)
            .max(f32::EPSILON);
        let rfac: Vec<f32> = vals
            .iter()
            .map(|v| {
                if style == PieStyle::Rose {
                    0.38 + 0.62 * (v / vmax).clamp(0.0, 1.0)
                } else {
                    1.0
                }
            })
            .collect();
        let scale_col = hsla(t.palette.text_muted);
        let guide_col = hsla(t.palette.border);
        let glow = glow_of(&t);
        let gap = if vals.len() > 1 { self.gap } else { 0.0 }; // px
        let paint = PiePaint {
            st: st_paint,
            fr,
            cols: cols.clone(),
            rfac,
            hover,
            sel,
            ring,
            motion,
            gap,
            glow,
            hud,
            soft,
            style,
            hole,
            size,
            explode,
            reserve,
            start,
            sweep,
            guide_col,
            scale_col,
            outline: {
                let cs = t.chart_style();
                cs.mark_outline.map(|c| (hsla(c), cs.mark_outline_width))
            },
        };
        let cvs = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, cx| paint.paint(b, w, cx),
        )
        .absolute()
        .size_full();
        let input = std::rc::Rc::new(PieInput {
            st: st_move,
            fr: fr_hit,
            size,
            reserve,
            hole,
            controlled,
            sel,
            on_select,
        });
        // A top half-ring (arc 0.75 + 0.5) only needs the upper half of the box.
        let top_half = self.sweep <= 0.5 && (self.start - 0.75).abs() < 0.01;
        let disc_h = if top_half { size / 2.0 + 2.0 } else { size };
        let mut disc = div()
            .id(key)
            .relative()
            .w(px(size))
            .h(px(disc_h))
            .overflow_hidden()
            .child(cvs)
            .on_mouse_move({
                let input = input.clone();
                move |e, w, cx| input.moved(e.position, w, cx)
            })
            .on_mouse_down(gpui::MouseButton::Left, {
                let input = input.clone();
                move |e, w, cx| input.clicked(e.position, w, cx)
            })
            .on_hover(move |over, w, cx| input.hover_changed(*over, w, cx));
        if let (Some(txt), true) = (self.center.clone(), hole > 0.0) {
            disc = disc.child(center_overlay(&t, txt, size, top_half));
        }
        let total: f32 = vals.iter().sum();
        let mut root = div()
            .flex()
            .items_center()
            .gap(px(t.spacing.lg))
            .child(disc);
        if self.legend {
            root = root.child(legend(&t, &self.slices, &cols, total));
        }
        root
    }
}

crate::entry_render!(PieChart);

#[cfg(test)]
mod tests {
    #[test]
    fn clicking_the_selected_slice_clears_it() {
        assert_eq!(super::next_selection(None, Some(2)), Some(2));
        assert_eq!(super::next_selection(Some(2), Some(2)), None);
        assert_eq!(super::next_selection(Some(1), Some(2)), Some(2));
        assert_eq!(super::next_selection(Some(1), None), None);
    }

    use super::*;

    /// A pie of a quarter, a quarter and a half, 0 at the top and clockwise.
    const SLICES: [(f32, f32); 3] = [(0.0, 0.25), (0.25, 0.5), (0.5, 1.0)];

    #[test]
    fn quadrants_hit_the_slice_there() {
        // Right of centre → the first slice's far edge… (turn 0.25 starts the second).
        assert_eq!(pie_hit(10.0, -10.0, 50.0, 0.0, &SLICES), Some(0)); // top right
        assert_eq!(pie_hit(10.0, 10.0, 50.0, 0.0, &SLICES), Some(1)); // bottom right
        assert_eq!(pie_hit(-10.0, 10.0, 50.0, 0.0, &SLICES), Some(2)); // bottom left
        assert_eq!(pie_hit(-10.0, -10.0, 50.0, 0.0, &SLICES), Some(2)); // top left
    }

    #[test]
    fn outside_the_disc_or_inside_the_hole_hits_nothing() {
        assert_eq!(pie_hit(200.0, 0.0, 50.0, 0.0, &SLICES), None);
        assert_eq!(pie_hit(1.0, 1.0, 50.0, 0.5, &SLICES), None);
        // A hovered slice pops out, so the pointer may sit PIE_POP beyond the radius.
        assert!(pie_hit(50.0 + PIE_POP - 1.0, -1.0, 50.0, 0.0, &SLICES).is_some());
    }

    #[test]
    fn a_partial_donut_that_wraps_past_one_is_still_hit() {
        // Starts at 0.75 (left) and sweeps half a turn, ending at 1.25 = the right side.
        let arc = [(0.75, 1.25)];
        assert_eq!(pie_hit(-30.0, 0.0, 50.0, 0.3, &arc), Some(0)); // left
        assert_eq!(pie_hit(30.0, -1.0, 50.0, 0.3, &arc), Some(0)); // just right of top
        assert_eq!(pie_hit(0.0, 30.0, 50.0, 0.3, &arc), None); // bottom is outside the arc
    }
}
