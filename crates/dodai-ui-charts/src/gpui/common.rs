//! Charts drawn on gpui's canvas / flex layout: [`LineChart`], [`BarChart`],
//! [`PieChart`] (+ donut), [`Sparkline`], [`RadarChart`],
//! [`ScatterPlot`], [`Heatmap`] and the shared [`Legend`].
//!
//! Colours come from `Palette::series`; lines glow when the theme has glow
//! effects; bars use the theme's shape (chamfer on cyberpunk).

use dodai_ui_core::{Color, Theme, Tone};
use dodai_ui_gpui::text::first_family;
use dodai_ui_gpui::{hsla, theme_of};
use gpui::{
    div, px, App, Hsla, IntoElement, ParentElement, Pixels, RenderOnce, SharedString, Styled,
    Window,
};
use std::rc::Rc;

pub use crate::gpui::bar::*;
pub use crate::gpui::grid::*;
pub use crate::gpui::line::*;
pub use crate::gpui::pie::*;
pub use crate::gpui::radar::*;
pub use crate::gpui::spark::*;

// ───────────────────────────── pure helpers ─────────────────────────────

pub(crate) use crate::trim;
pub use crate::{
    fmt_num, marker_points, nice_ticks, norm, pie_fractions, polar, smooth_points, step_points,
    turn_arc, Marker,
};

pub(crate) use dodai_ui_gpui::paint::{fill, fill_batch, stroke, stroke_batch};
pub use dodai_ui_gpui::plot::{chart_glow, set_chart_glow, ChartGlow};
pub(crate) use dodai_ui_gpui::plot::{glow_of, glow_stroke, rect};

/// One gridline from `a` to `b` in the design's chart style (solid or dashed).
pub(crate) fn grid_line(
    w: &mut Window,
    o: gpui::Point<Pixels>,
    a: (f32, f32),
    b: (f32, f32),
    s: &dodai_ui_core::ChartStyle,
) {
    let c = hsla(s.grid_color);
    if !s.grid_dashed {
        stroke(w, o, &[a, b], s.grid_width, c, false);
        return;
    }
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len = (dx * dx + dy * dy).sqrt();
    let dash = (s.grid_width * 4.0).max(4.0);
    let (ux, uy) = if len > 0.0 {
        (dx / len, dy / len)
    } else {
        (0.0, 0.0)
    };
    let mut at = 0.0;
    while at < len {
        let end = (at + dash).min(len);
        let seg = [
            (a.0 + ux * at, a.1 + uy * at),
            (a.0 + ux * end, a.1 + uy * end),
        ];
        stroke(w, o, &seg, s.grid_width, c, false);
        at += dash * 2.0;
    }
}

// ─────────────────────────────── data types ───────────────────────────────

/// Named numeric series; colour defaults to `palette.series[index]`.
#[derive(Clone)]
pub struct Series {
    /// Series name (legend entry).
    pub name: SharedString,
    /// One value per category or x position.
    pub values: Vec<f32>,
    /// Explicit colour; `None` takes the next palette colour.
    pub color: Option<Color>,
    /// A live window: its samples replace `values` every frame.
    pub(crate) live: Option<dodai_ui_gpui::live::Live>,
    /// Lower and upper edge per value (see [`Series::band`]).
    pub(crate) band: Option<(Vec<f32>, Vec<f32>)>,
}

/// NaN / ±inf become 0: hostile data must never reach the tessellator.
pub(crate) fn finite_or_zero(v: f32) -> f32 {
    if v.is_finite() {
        v
    } else {
        0.0
    }
}

impl Series {
    /// Create a new [`Series`].
    pub fn new(name: impl Into<SharedString>, values: impl IntoIterator<Item = f32>) -> Self {
        Self {
            name: name.into(),
            values: values.into_iter().map(finite_or_zero).collect(),
            color: None,
            live: None,
            band: None,
        }
    }
    /// A band around the line, one `lo`/`hi` pair per value: a confidence interval, the
    /// min..max of a period or an error range. A [`crate::gpui::line::LineChart`] shades it
    /// under the line and fits its axis to it. Pairs given the wrong way round are swapped.
    pub fn band(
        mut self,
        lo: impl IntoIterator<Item = f32>,
        hi: impl IntoIterator<Item = f32>,
    ) -> Self {
        let (lo, hi): (Vec<f32>, Vec<f32>) = lo
            .into_iter()
            .zip(hi)
            .map(|(a, b)| {
                let (a, b) = (finite_or_zero(a), finite_or_zero(b));
                (a.min(b), a.max(b))
            })
            .unzip();
        self.band = Some((lo, hi));
        self
    }
    /// A series fed by a [`dodai_ui_gpui::Live`] window: it shows the newest samples and, in a
    /// [`crate::gpui::line::LineChart`], slides left between them.
    pub fn live(name: impl Into<SharedString>, window: &dodai_ui_gpui::live::Live) -> Self {
        let mut s = Self::new(name, window.values());
        s.live = Some(window.clone());
        s
    }
    /// The scroll fraction of this series' live window, if it has one.
    pub(crate) fn scroll(&self) -> Option<f32> {
        self.live.as_ref().map(dodai_ui_gpui::live::Live::scroll)
    }
    /// Colour (default: the theme's).
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }
}

pub(crate) fn series_color(t: &Theme, s: &Series, i: usize) -> Color {
    s.color.unwrap_or_else(|| t.palette.series_at(i))
}

pub(crate) fn y_range(
    series: &[Series],
    zero: bool,
    fixed: Option<(f32, f32)>,
) -> (f32, f32, Vec<f32>) {
    if let Some((lo, hi)) = fixed {
        let ticks = nice_ticks(lo, hi, 4);
        return (
            lo,
            hi,
            ticks.into_iter().filter(|v| *v >= lo && *v <= hi).collect(),
        );
    }
    let vals = series.iter().flat_map(|s| {
        let band = s.band.iter().flat_map(|(lo, hi)| lo.iter().chain(hi));
        s.values.iter().chain(band).copied()
    });
    let (mut lo, hi) = crate::extent(vals).unwrap_or((0.0, 1.0));
    if zero {
        lo = lo.min(0.0);
    }
    let ticks = nice_ticks(lo, hi, 4);
    (
        *ticks.first().unwrap_or(&lo),
        *ticks.last().unwrap_or(&hi),
        ticks,
    )
}

// ─────────────────────────────── Legend ───────────────────────────────

/// Which series of a chart the user switched off by clicking its legend entry.
pub(crate) type Hidden = gpui::Entity<Vec<bool>>;

/// Pin every series to its palette colour (so hiding one does not recolour the rest), then drop
/// the hidden ones. Returns the visible series, the legend state and the full pinned list.
pub(crate) fn toggled(
    window: &mut Window,
    cx: &mut App,
    id: &str,
    all: Vec<Series>,
) -> (Vec<Series>, Hidden, Vec<Series>) {
    let t = theme_of(cx);
    let hidden: Hidden =
        window.use_keyed_state(SharedString::from(format!("{id}-hidden")), cx, |_, _| {
            Vec::new()
        });
    let pinned: Vec<Series> = all
        .into_iter()
        .enumerate()
        .map(|(i, mut s)| {
            s.color = Some(series_color(&t, &s, i));
            s
        })
        .collect();
    let off = hidden.read(cx).clone();
    let shown = pinned
        .iter()
        .enumerate()
        .filter(|(i, _)| !off.get(*i).copied().unwrap_or(false))
        .map(|(_, s)| s.clone())
        .collect();
    (shown, hidden, pinned)
}

/// Colour swatch + name row, shared by all multi-series charts.
#[derive(IntoElement)]
pub struct Legend {
    items: Vec<(SharedString, Color)>,
    toggle: Option<(SharedString, Hidden)>,
}

impl Legend {
    /// Create a new [`Legend`].
    pub fn new(items: impl IntoIterator<Item = (SharedString, Color)>) -> Self {
        Self {
            items: items.into_iter().collect(),
            toggle: None,
        }
    }
    /// Make the entries clickable: a click hides or shows that series (state in `hidden`).
    pub(crate) fn toggle(mut self, id: impl Into<SharedString>, hidden: Hidden) -> Self {
        self.toggle = Some((id.into(), hidden));
        self
    }
    /// Legend for `series` using palette colours.
    pub fn of_series(t: &Theme, series: &[Series]) -> Self {
        Self::new(
            series
                .iter()
                .enumerate()
                .map(|(i, s)| (s.name.clone(), series_color(t, s, i))),
        )
    }
}

impl RenderOnce for Legend {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let motion = dodai_ui_gpui::motion::current(cx);
        dodai_ui_gpui::motion::entry(self.build(cx), "Legend", motion)
    }
}

impl Legend {
    fn build(self, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        use gpui::{InteractiveElement as _, StatefulInteractiveElement as _};
        let toggle = self.toggle;
        let off: Vec<bool> = toggle
            .as_ref()
            .map(|(_, h)| h.read(cx).clone())
            .unwrap_or_default();
        div()
            .flex()
            .flex_wrap()
            .gap(px(t.spacing.md))
            .font_family(first_family(t.typography.body))
            .text_size(px(t.typography.size_xs))
            .text_color(hsla(t.palette.text_muted))
            .children(self.items.into_iter().enumerate().map(|(i, (n, c))| {
                let hidden = off.get(i).copied().unwrap_or(false);
                let mut row = div()
                    .id((
                        toggle
                            .as_ref()
                            .map(|(id, _)| id.clone())
                            .unwrap_or_else(|| "legend".into()),
                        i,
                    ))
                    .flex()
                    .items_center()
                    .gap(px(t.spacing.xs))
                    .opacity(if hidden { 0.4 } else { 1.0 });
                if let Some((_, h)) = toggle.clone() {
                    row = row.cursor_pointer().on_click(move |_, _, cx| {
                        h.update(cx, |v, cx| {
                            if v.len() <= i {
                                v.resize(i + 1, false);
                            }
                            v[i] = !v[i];
                            cx.notify();
                        })
                    });
                }
                row.child(
                    div()
                        .relative()
                        .w(px(t.sizes().legend_swatch))
                        .h(px(t.sizes().legend_swatch))
                        .bg(hsla(c))
                        .children(hatch_overlay(&t, i)),
                )
                .child(n)
            }))
    }
}

// ─────────────────────────── axes frame (shared) ───────────────────────────

pub(crate) fn label_style(t: &Theme, d: gpui::Div) -> gpui::Div {
    d.font_family(first_family(t.typography.mono))
        .text_size(px(t.typography.size_xs))
        .text_color(hsla(t.palette.text_muted))
}

/// Where a label may go: a point and the direction it grows in (a unit vector).
pub(crate) type Spot = ((f32, f32), (f32, f32));

/// Chart label text painted straight onto a canvas, for labels that sit on shapes laid out at
/// paint time (inside a bubble, a square).
#[derive(Clone)]
pub(crate) struct CanvasText {
    family: SharedString,
    size: f32,
    weight: gpui::FontWeight,
}

impl CanvasText {
    /// The chart label font of `t`.
    pub(crate) fn of(t: &Theme) -> Self {
        Self {
            family: first_family(t.typography.mono),
            size: t.typography.size_xs,
            weight: gpui::FontWeight::NORMAL,
        }
    }

    /// The display (heading) font of `t` at `size` px, for labels that carry a magnitude.
    pub(crate) fn display(t: &Theme, size: f32) -> Self {
        Self {
            family: first_family(t.typography.display),
            size,
            weight: gpui::FontWeight::NORMAL,
        }
    }

    /// The same font in bold.
    pub(crate) fn bold(mut self) -> Self {
        self.weight = gpui::FontWeight::BOLD;
        self
    }

    /// Width and line height of `text`.
    pub(crate) fn measure(&self, w: &mut Window, text: &str) -> (f32, f32) {
        let line = self.shape(w, text, gpui::transparent_black());
        (line.width.into(), self.size_line())
    }

    /// Height of one line of label text.
    pub(crate) fn size_line(&self) -> f32 {
        self.size * 1.3
    }

    /// `text` laid out in the label font and colour.
    fn shape(&self, w: &mut Window, text: &str, color: Hsla) -> gpui::ShapedLine {
        let run = gpui::TextRun {
            len: text.len(),
            font: gpui::Font {
                weight: self.weight,
                ..gpui::font(self.family.clone())
            },
            color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        w.text_system().shape_line(
            SharedString::from(text.to_string()),
            px(self.size),
            &[run],
            None,
        )
    }

    /// Paints `text` just beyond `at` in the direction `(ux, uy)` (a unit vector): the label
    /// grows away from the point, as names around a ring do.
    pub(crate) fn beside(
        &self,
        w: &mut Window,
        cx: &mut gpui::App,
        text: &str,
        (o, at): (gpui::Point<Pixels>, (f32, f32)),
        (ux, uy): (f32, f32),
        color: Hsla,
    ) {
        if text.is_empty() {
            return;
        }
        let line = self.shape(w, text, color);
        let (lw, lh): (f32, f32) = (line.width.into(), self.size_line());
        let (x, y) = (
            at.0 + ux * lw / 2.0 - lw / 2.0,
            at.1 + uy * lh / 2.0 - lh / 2.0,
        );
        let _ = line.paint(gpui::point(o.x + px(x), o.y + px(y)), px(lh), w, cx);
    }

    /// Like [`Self::beside`], but only where the label's box misses every box in `taken`
    /// (then adds it): crowded labels drop out instead of overprinting. Returns whether it
    /// was painted.
    pub(crate) fn beside_free(
        &self,
        w: &mut Window,
        cx: &mut gpui::App,
        (text, color): (&str, Hsla),
        (o, at): (gpui::Point<Pixels>, (f32, f32)),
        (ux, uy): (f32, f32),
        taken: &mut Vec<[f32; 4]>,
    ) -> bool {
        self.place_free(w, cx, (text, color), o, &[(at, (ux, uy))], taken)
            .is_some()
    }

    /// Paints `text` at the first of `spots` (each a point and a direction, as for
    /// [`Self::beside`]) where its box misses every box in `taken`, adds the box and returns it.
    pub(crate) fn place_free(
        &self,
        w: &mut Window,
        cx: &mut gpui::App,
        (text, color): (&str, Hsla),
        o: gpui::Point<Pixels>,
        spots: &[Spot],
        taken: &mut Vec<[f32; 4]>,
    ) -> Option<[f32; 4]> {
        if text.is_empty() {
            return None;
        }
        let line = self.shape(w, text, color);
        let (lw, lh): (f32, f32) = (line.width.into(), self.size_line());
        let r = spots.iter().find_map(|&(at, (ux, uy))| {
            let (x, y) = (
                at.0 + ux * lw / 2.0 - lw / 2.0,
                at.1 + uy * lh / 2.0 - lh / 2.0,
            );
            let r = [x - 2.0, y, x + lw + 2.0, y + lh];
            let free = !taken.iter().any(|t| overlaps(&r, t));
            free.then_some(r)
        })?;
        taken.push(r);
        let _ = line.paint(
            gpui::point(o.x + px(r[0] + 2.0), o.y + px(r[1])),
            px(lh),
            w,
            cx,
        );
        Some(r)
    }

    /// Paints `text` centred on `at` (a canvas origin and a point from it) when it is at most
    /// `max_w` wide; returns whether it fit.
    pub(crate) fn centred(
        &self,
        w: &mut Window,
        cx: &mut gpui::App,
        text: &str,
        (o, at): (gpui::Point<Pixels>, (f32, f32)),
        max_w: f32,
        color: Hsla,
    ) -> bool {
        if text.is_empty() || max_w <= 0.0 {
            return false;
        }
        let line = self.shape(w, text, color);
        let lw: f32 = line.width.into();
        if lw > max_w {
            return false;
        }
        let lh = self.size_line();
        let _ = line.paint(
            gpui::point(o.x + px(at.0 - lw / 2.0), o.y + px(at.1 - lh / 2.0)),
            px(lh),
            w,
            cx,
        );
        true
    }
}

/// Colour slots for items that may name a group: groups take slots in order of first
/// appearance, ungrouped items one each after them. Also returns the group names.
pub(crate) fn group_slots<'a>(
    groups: impl IntoIterator<Item = Option<&'a SharedString>>,
) -> (Vec<usize>, Vec<SharedString>) {
    let groups: Vec<Option<&SharedString>> = groups.into_iter().collect();
    let mut names: Vec<SharedString> = Vec::new();
    for g in groups.iter().flatten() {
        if !names.contains(g) {
            names.push((*g).clone());
        }
    }
    let mut lone = names.len();
    let slots = groups
        .iter()
        .map(|g| match g {
            Some(g) => names.iter().position(|n| n == *g).unwrap_or(0),
            None => {
                lone += 1;
                lone - 1
            }
        })
        .collect();
    (slots, names)
}

/// The groups of a grouped chart: a colour slot per item (see [`group_slots`]) and, with the
/// filter on, a legend whose entries hide and show their group.
pub(crate) struct Groups {
    pub(crate) slots: Vec<usize>,
    pub(crate) names: Vec<SharedString>,
    toggle: Option<(SharedString, Hidden)>,
    off: Vec<bool>,
}

impl Groups {
    pub(crate) fn new<'a>(
        window: &mut Window,
        cx: &mut App,
        follow: &dodai_ui_gpui::follow::Follow,
        groups: impl IntoIterator<Item = Option<&'a SharedString>>,
        filter: bool,
    ) -> Self {
        let (slots, names) = group_slots(groups);
        let toggle = (filter && names.len() > 1).then(|| {
            let id = follow.sub("groups").element_id(window, cx);
            let hidden: Hidden = window.with_id(id.clone(), |w| {
                w.use_keyed_state("hidden", cx, |_, _| Vec::<bool>::new())
            });
            (SharedString::from(format!("groups-{id:?}")), hidden)
        });
        let off = toggle
            .as_ref()
            .map(|(_, h)| h.read(cx).clone())
            .unwrap_or_default();
        Self {
            slots,
            names,
            toggle,
            off,
        }
    }

    /// Whether item `i` is shown: its group isn't hidden.
    pub(crate) fn shown(&self, i: usize) -> bool {
        let slot = self.slots.get(i).copied().unwrap_or(usize::MAX);
        !self.off.get(slot).copied().unwrap_or(false)
    }

    /// The shown items of `items`, with their colour slots.
    pub(crate) fn keep<T>(&self, items: Vec<T>) -> (Vec<T>, Vec<usize>) {
        items
            .into_iter()
            .enumerate()
            .filter(|(i, _)| self.shown(*i))
            .map(|(i, it)| (it, self.slots[i]))
            .unzip()
    }

    /// The legend: nothing without groups, clickable with the filter on.
    pub(crate) fn legend(&self, t: &Theme) -> Option<Legend> {
        (!self.names.is_empty()).then(|| {
            let lg = Legend::new(
                self.names
                    .iter()
                    .enumerate()
                    .map(|(i, n)| (n.clone(), t.palette.series_at(i))),
            );
            match &self.toggle {
                Some((id, hidden)) => lg.toggle(id.clone(), hidden.clone()),
                None => lg,
            }
        })
    }

    /// The readout's hint, `base` plus how to filter when the legend is clickable.
    pub(crate) fn hint(&self, base: &str) -> String {
        match self.toggle {
            Some(_) => format!("{base} · click a group to hide it"),
            None => base.to_string(),
        }
    }
}

/// A canvas that records its bounds into `bounds` (for [`Focus::pointer`](crate::gpui::focus::Focus::pointer))
/// and then paints with `paint`.
pub(crate) fn tracked_canvas(
    bounds: &Rc<std::cell::Cell<gpui::Bounds<Pixels>>>,
    paint: impl FnOnce(gpui::Bounds<Pixels>, &mut gpui::Window, &mut App) + 'static,
) -> impl IntoElement + Styled {
    let store = bounds.clone();
    gpui::canvas(
        |_, _, _| (),
        move |b, _, w, cx| {
            store.set(b);
            paint(b, w, cx)
        },
    )
}

/// The positioned, full-width box of `height` px a chart's canvas sits in.
pub(crate) fn plot_box(
    id: impl Into<gpui::ElementId>,
    height: f32,
    child: impl IntoElement,
) -> gpui::Stateful<gpui::Div> {
    use gpui::InteractiveElement as _;
    div().id(id).relative().w_full().h(px(height)).child(child)
}

/// Whether boxes `a` and `b` (`[left, top, right, bottom]`) overlap; touching edges don't.
pub(crate) fn overlaps(a: &[f32; 4], b: &[f32; 4]) -> bool {
    a[0] < b[2] && b[0] < a[2] && a[1] < b[3] && b[1] < a[3]
}

/// Colour `i` of a long list: the palette's series, then lighter and darker turns of it, so
/// more series than the palette holds stay apart.
pub(crate) fn many_series(t: &Theme, i: usize) -> Color {
    let n = t.palette.series.len().max(1);
    let c = t.palette.series_at(i);
    match (i / n) % 3 {
        0 => c,
        1 => c.lighten(0.3),
        _ => c.darken(0.25),
    }
}

/// Width of a column of row labels: room for the longest one (estimated from the label
/// font's average advance), never narrower than the large axis gutter nor wider than 220 px.
pub(crate) fn label_column<S: AsRef<str>>(t: &Theme, labels: impl IntoIterator<Item = S>) -> f32 {
    let chars = labels
        .into_iter()
        .map(|l| l.as_ref().chars().count())
        .max()
        .unwrap_or(0);
    (chars as f32 * t.typography.size_xs * 0.62 + t.spacing.xs)
        .clamp(t.sizes().axis_gutter_lg, 220.0)
}

/// Value labels spread evenly under (or beside) a plot whose axis runs from the first tick to
/// the last, as `nice_ticks` returns them.
pub(crate) fn tick_row(t: &Theme, ticks: &[f32], fmt: &Option<NumFmt>) -> gpui::Div {
    label_style(t, div())
        .flex()
        .justify_between()
        .children(ticks.iter().map(|v| div().child(fmt_with(fmt, *v))))
}

/// Y-axis labels (top = max) + plot + optional X labels underneath.
pub(crate) fn framed(
    t: &Theme,
    y_labels: Vec<String>,
    x_labels: Vec<SharedString>,
    plot: gpui::Div,
) -> gpui::Div {
    let row = (!x_labels.is_empty()).then(|| slot_labels(t, x_labels));
    framed_row(t, y_labels, row, plot)
}

/// Names under equal slots (one per bar, point or column), painted on one canvas: each centred
/// in its slot, kept inside the row, skipped when it would touch the one before. Painting keeps
/// dozens of labels out of layout, which a scrolling page redoes for every visible chart.
pub(crate) fn slot_labels(t: &Theme, labels: Vec<SharedString>) -> gpui::Div {
    let text = CanvasText::of(t);
    let (color, gap) = (hsla(t.palette.text_muted), t.spacing.xs);
    div().w_full().h(px(text.size_line())).child(
        gpui::canvas(
            |_, _, _| (),
            move |b: gpui::Bounds<Pixels>, _, w, cx| {
                let (width, n) = (f32::from(b.size.width), labels.len() as f32);
                let mut free_from = f32::NEG_INFINITY;
                for (i, l) in labels.iter().enumerate() {
                    if l.is_empty() {
                        continue;
                    }
                    let line = text.shape(w, l, color);
                    let lw = f32::from(line.width);
                    let centre = (i as f32 + 0.5) / n * width;
                    let x = (centre - lw / 2.0).clamp(0.0, (width - lw).max(0.0));
                    if x < free_from {
                        continue;
                    }
                    free_from = x + lw + gap;
                    let at = gpui::point(b.origin.x + px(x), b.origin.y);
                    let _ = line.paint(at, px(text.size_line()), w, cx);
                }
            },
        )
        .size_full(),
    )
}

/// [`framed`] with a caller-built x-axis row (e.g. labels placed at arbitrary positions).
pub(crate) fn framed_row(
    t: &Theme,
    y_labels: Vec<String>,
    x_row: Option<gpui::Div>,
    plot: gpui::Div,
) -> gpui::Div {
    // At least the gutter, wider when a label needs it: labels never wrap.
    let ycol = label_style(t, div())
        .min_w(px(t.sizes().axis_gutter))
        .flex_none()
        .whitespace_nowrap()
        .flex()
        .flex_col()
        .justify_between()
        .items_end()
        .pr(px(t.spacing.xs))
        .children(y_labels.into_iter().rev().map(|l| div().child(l)));
    let mut right = div()
        .flex_1()
        .flex()
        .flex_col()
        .gap(px(t.spacing.xs))
        .child(plot);
    if let Some(row) = x_row {
        right = right.child(row);
    }
    div().flex().w_full().child(ycol).child(right)
}

/// Names for a chart's axes: the y title sits over the value labels (where a unit reads first),
/// the x title centred under the plot.
#[derive(Clone, Default)]
pub(crate) struct AxisTitles {
    pub(crate) x: Option<SharedString>,
    pub(crate) y: Option<SharedString>,
}

impl AxisTitles {
    /// `body` (a framed plot whose label gutter is `gutter` px wide) with the titles around it.
    pub(crate) fn around(&self, t: &Theme, gutter: f32, body: gpui::Div) -> gpui::Div {
        if self.x.is_none() && self.y.is_none() {
            return body;
        }
        let caption = |s: SharedString| {
            label_style(t, div())
                .font_weight(gpui::FontWeight::MEDIUM)
                .child(s)
        };
        let mut out = div().flex().flex_col().gap(px(t.spacing.xs)).w_full();
        if let Some(y) = self.y.clone() {
            out = out.child(caption(y));
        }
        out = out.child(body);
        if let Some(x) = self.x.clone() {
            out = out.child(
                div()
                    .flex()
                    .justify_center()
                    .pl(px(gutter))
                    .child(caption(x)),
            );
        }
        out
    }
}

/// Number formatter shared by chart configuration.
pub type NumFmt = Rc<dyn Fn(f32) -> String>;

pub(crate) fn fmt_with(f: &Option<NumFmt>, v: f32) -> String {
    f.as_ref().map_or_else(|| fmt_num(v), |f| f(v))
}

/// Optional chart title above `body`.
pub(crate) fn titled(t: &Theme, title: Option<SharedString>, body: gpui::Div) -> gpui::Div {
    match title {
        None => body,
        Some(s) => div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.sm))
            .w_full()
            .child(
                div()
                    .font_family(first_family(t.typography.display))
                    .text_size(px(t.typography.size_md))
                    .text_color(hsla(t.palette.text))
                    .child(s),
            )
            .child(body),
    }
}

pub(crate) fn up_color(t: &Theme) -> Hsla {
    hsla(t.palette.tone(Tone::Success))
}
pub(crate) fn down_color(t: &Theme) -> Hsla {
    hsla(t.palette.tone(Tone::Danger))
}

macro_rules! chart_cfg {
    ($ty:ident) => {
        impl $ty {
            /// Chart height in px.
            pub fn height(mut self, h: f32) -> Self {
                self.height = h;
                self
            }
            /// Title above the chart.
            pub fn title(mut self, t: impl Into<SharedString>) -> Self {
                self.title = Some(t.into());
                self
            }
            /// Custom number formatter for labels and tooltips.
            pub fn format(mut self, f: impl Fn(f32) -> String + 'static) -> Self {
                self.fmt = Some(Rc::new(f));
                self
            }
        }
    };
}

pub(crate) fn labels_of<S: Into<SharedString>>(
    l: impl IntoIterator<Item = S>,
) -> Vec<SharedString> {
    l.into_iter().map(Into::into).collect()
}

pub(crate) use chart_cfg;

/// A chart's series with their values followed. Live series take their window's newest samples
/// and are not eased: they already move continuously.
pub fn follow_series(
    w: &mut gpui::Window,
    cx: &mut gpui::App,
    f: &dodai_ui_gpui::follow::Follow,
    series: &[Series],
) -> Vec<Series> {
    let series: Vec<Series> = series
        .iter()
        .map(|s| {
            let mut s = s.clone();
            if let Some(l) = &s.live {
                s.values = l.values();
            }
            s
        })
        .collect();
    // Each series has its own springs: with one flat list, adding a point or a series moved every
    // later value onto the wrong spring and the whole chart jumped.
    series
        .into_iter()
        .enumerate()
        .map(|(i, mut s)| {
            if s.live.is_none() {
                s.values =
                    dodai_ui_gpui::follow::follow_all(w, cx, &f.sub(&format!("s{i}")), &s.values);
                if let Some((lo, hi)) = &mut s.band {
                    *lo = dodai_ui_gpui::follow::follow_all(w, cx, &f.sub(&format!("s{i}-lo")), lo);
                    *hi = dodai_ui_gpui::follow::follow_all(w, cx, &f.sub(&format!("s{i}-hi")), hi);
                }
            }
            s
        })
        .collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn spark_columns_shapes() {
        let v = [1.0, 5.0, 3.0, 9.0];
        let bars = spark_columns(&v, 40.0, 20.0, false);
        assert_eq!(bars.len(), 4);
        // Tallest value reaches the top; the lowest keeps a visible stub.
        assert!(bars[3].1 < 0.5 && bars[0].3 - bars[0].1 > 2.0);
        assert!(
            bars.windows(2).all(|w| w[0].2 <= w[1].0 + 0.01),
            "no overlap"
        );
        let wl = spark_columns(&v, 40.0, 20.0, true);
        // mean 4.5: values 1,3 lose (below the midline), 5,9 win (above).
        assert!(wl[0].1 > 10.0 && wl[2].1 > 10.0);
        assert!(wl[1].3 < 10.0 && wl[3].3 < 10.0);
        assert!(spark_columns(&[], 10.0, 10.0, false).is_empty());
    }

    #[test]
    fn pie_hit_wraps_past_one_turn() {
        // Half donut starting at the left (0.75) running over the top to 1.25.
        let fr = [(0.75, 1.0), (1.0, 1.25)];
        assert_eq!(pie_hit(0.0, -30.0, 50.0, 0.4, &fr), Some(1)); // just right of top
        assert_eq!(pie_hit(-30.0, -1.0, 50.0, 0.4, &fr), Some(0)); // upper left
        assert_eq!(pie_hit(0.0, 30.0, 50.0, 0.4, &fr), None); // empty lower half
    }

    #[test]
    fn pie_hit_finds_slice() {
        let fr = [(0.0, 0.5), (0.5, 1.0)];
        assert_eq!(pie_hit(10.0, 0.0, 50.0, 0.0, &fr), Some(0)); // right = 25%
        assert_eq!(pie_hit(-10.0, 0.0, 50.0, 0.0, &fr), Some(1));
        assert_eq!(pie_hit(0.0, 0.0, 50.0, 0.4, &fr), None); // donut hole
        assert_eq!(pie_hit(200.0, 0.0, 50.0, 0.0, &fr), None);
    }

    use super::*;

    #[test]
    fn hatch_lines_stay_inside_the_polygon() {
        // An L shape: the notch must stay empty.
        let l = [
            (0.0, 0.0),
            (30.0, 0.0),
            (30.0, 10.0),
            (10.0, 10.0),
            (10.0, 30.0),
            (0.0, 30.0),
        ];
        let segs = hatch_segments(&l, Hatch::Cross);
        assert!(!segs.is_empty());
        for [a, b] in &segs {
            for p in [a, b] {
                assert!(p.0 >= -0.01 && p.0 <= 30.01 && p.1 >= -0.01 && p.1 <= 30.01);
            }
            let mid = ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
            assert!(
                !(mid.0 > 10.5 && mid.1 > 10.5),
                "a stroke crosses the notch: {mid:?}"
            );
        }
        assert!(hatch_segments(&l[..2], Hatch::Slash).is_empty());
        let slash = hatch_segments(&l, Hatch::Slash).len();
        let both = hatch_segments(&l, Hatch::Cross).len();
        assert!(both > slash);
    }

    #[test]
    fn bar_max_stacked_vs_grouped() {
        let s = [Series::new("a", [1.0, 4.0]), Series::new("b", [3.0, 1.0])];
        assert_eq!(bar_max(&s, true), 5.0);
        assert_eq!(bar_max(&s, false), 4.0);
    }

    #[test]
    fn hatching_cycles_and_the_first_series_stays_solid() {
        assert_eq!(hatch_of(0), None);
        assert_eq!(hatch_of(1), Some(Hatch::Slash));
        assert_eq!(hatch_of(2), Some(Hatch::Backslash));
        assert_eq!(hatch_of(3), Some(Hatch::Cross));
        assert_eq!(hatch_of(4), None);
    }
}

thread_local! {
    static PATTERNS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Tell every chart to mark its series by pattern as well as colour (dashed lines, hatched
/// bars and legend swatches) so they stay apart for colour-blind readers, and repaint. Off by
/// default; a chart's own `.patterns(..)` (where it has one) turns it on for that chart only.
pub fn set_patterns(cx: &mut App, on: bool) {
    PATTERNS.with(|c| c.set(on));
    cx.refresh_windows();
}

/// Whether charts mark series by pattern too (see [`set_patterns`]).
pub fn patterns() -> bool {
    PATTERNS.with(std::cell::Cell::get)
}

/// A hatching drawn over a mark to tell series apart without colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hatch {
    /// Rising diagonals.
    Slash,
    /// Falling diagonals.
    Backslash,
    /// Both.
    Cross,
}

/// The hatching of series `i`: the first is solid, the rest cycle through [`Hatch`] (matching
/// the line dashes of the same index).
pub fn hatch_of(i: usize) -> Option<Hatch> {
    match i % 4 {
        0 => None,
        1 => Some(Hatch::Slash),
        2 => Some(Hatch::Backslash),
        _ => Some(Hatch::Cross),
    }
}

/// The colour hatch lines are drawn in over a series fill: the background, so they cut the fill.
pub(crate) fn hatch_ink(t: &Theme) -> Hsla {
    dodai_ui_gpui::paint::scale_alpha(hsla(t.palette.background), 0.6)
}

/// Draws a series' hatch over its fill when patterns are on; build one outside the paint closure.
#[derive(Clone, Copy)]
pub(crate) struct Hatcher(Option<Hsla>);

impl Hatcher {
    pub(crate) fn new(t: &Theme) -> Self {
        Self(patterns().then(|| hatch_ink(t)))
    }

    /// Hatches `poly` as series (or category) `i`; the first stays solid.
    pub(crate) fn over(
        &self,
        w: &mut Window,
        o: gpui::Point<Pixels>,
        poly: &[(f32, f32)],
        i: usize,
    ) {
        if let (Some(ink), Some(kind)) = (self.0, hatch_of(i)) {
            hatch_poly(w, o, poly, ink, kind);
        }
    }
}

/// The hatch segments of `kind` clipped to a polygon (any simple shape, even-odd). The lines sit
/// on a grid fixed in the polygon's own coordinates, so neighbouring pieces of one shape line up.
pub(crate) fn hatch_segments(poly: &[(f32, f32)], kind: Hatch) -> Vec<[(f32, f32); 2]> {
    let mut out = Vec::new();
    if poly.len() < 3 || poly.iter().any(|p| !(p.0.is_finite() && p.1.is_finite())) {
        return out;
    }
    for (sign, on) in [
        (1.0_f32, kind != Hatch::Backslash),
        (-1.0, kind != Hatch::Slash),
    ] {
        if !on {
            continue;
        }
        // Lines x + sign·y = k.
        let ks: Vec<f32> = poly.iter().map(|p| p.0 + sign * p.1).collect();
        let lo = ks.iter().copied().fold(f32::INFINITY, f32::min);
        let hi = ks.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let mut k = (lo / HATCH_STEP).ceil() * HATCH_STEP;
        while k < hi {
            let mut xs: Vec<(f32, f32)> = Vec::new();
            for (i, p) in poly.iter().enumerate() {
                let q = poly[(i + 1) % poly.len()];
                let (fp, fq) = (p.0 + sign * p.1 - k, q.0 + sign * q.1 - k);
                if (fp < 0.0) != (fq < 0.0) {
                    let u = fp / (fp - fq);
                    let x = p.0 + (q.0 - p.0) * u;
                    xs.push((x, (k - x) * sign));
                }
            }
            xs.sort_by(|a, b| a.0.total_cmp(&b.0));
            out.extend(xs.chunks_exact(2).map(|pair| [pair[0], pair[1]]));
            k += HATCH_STEP;
        }
    }
    out
}

/// [`hatch_segments`] drawn at `o` in `c`.
pub(crate) fn hatch_poly(
    w: &mut Window,
    o: gpui::Point<Pixels>,
    poly: &[(f32, f32)],
    c: Hsla,
    kind: Hatch,
) {
    for seg in hatch_segments(poly, kind) {
        dodai_ui_gpui::paint::stroke(w, o, &seg, 1.0, c, false);
    }
}

/// Hatching of `kind` over a `w × h` box at `o`, in `c`. Pure strokes: it needs no clipping.
pub(crate) fn hatch(
    w: &mut Window,
    o: gpui::Point<Pixels>,
    (bw, bh): (f32, f32),
    c: Hsla,
    kind: Hatch,
) {
    if !(bw.is_finite() && bh.is_finite() && bw > 0.0 && bh > 0.0) {
        return;
    }
    let step = HATCH_STEP;
    if kind != Hatch::Backslash {
        let mut k = -bh;
        while k < bw {
            // The line x = k + s, y = bh - s, kept inside 0..bw.
            let (s0, s1) = ((-k).max(0.0), bh.min(bw - k));
            if s1 > s0 {
                dodai_ui_gpui::paint::stroke(
                    w,
                    o,
                    &[(k + s0, bh - s0), (k + s1, bh - s1)],
                    1.0,
                    c,
                    false,
                );
            }
            k += step;
        }
    }
    if kind != Hatch::Slash {
        let mut k = -bh;
        while k < bw {
            // The line x = k + s, y = s, kept inside 0..bw.
            let (s0, s1) = ((-k).max(0.0), bh.min(bw - k));
            if s1 > s0 {
                dodai_ui_gpui::paint::stroke(w, o, &[(k + s0, s0), (k + s1, s1)], 1.0, c, false);
            }
            k += step;
        }
    }
}

/// Distance between hatch lines, px.
const HATCH_STEP: f32 = 6.0;

/// An overlay that hatches its parent for series `i` when patterns are on (else nothing).
pub(crate) fn hatch_overlay(t: &Theme, i: usize) -> Option<gpui::AnyElement> {
    let kind = hatch_of(i).filter(|_| patterns())?;
    let c = hatch_ink(t);
    Some(
        gpui::canvas(
            |_, _, _| (),
            move |b: gpui::Bounds<Pixels>, _, w, _| {
                hatch(
                    w,
                    b.origin,
                    (b.size.width.into(), b.size.height.into()),
                    c,
                    kind,
                );
            },
        )
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .into_any_element(),
    )
}
