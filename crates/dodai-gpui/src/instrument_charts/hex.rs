//! [`HexMap`] and [`RingStack`].

use super::*;

/// Honeycomb heat map: values `0..=1`, row-major, `cols` per row.
#[derive(IntoElement)]
pub struct HexMap {
    follow: crate::instruments_motion::Follow,
    glow: ChartGlow,
    values: Vec<f32>,
    cols: usize,
    w: f32,
    live: bool,
    caption: Option<SharedString>,
}

impl HexMap {
    /// The glow ("shadow") under the lines: the theme's own, none, or a radius.
    pub fn glow(mut self, g: ChartGlow) -> Self {
        self.glow = g;
        self
    }
    /// Create a new [`HexMap`]; changed values glide.
    #[track_caller]
    pub fn new(cols: usize, values: impl IntoIterator<Item = f32>) -> Self {
        Self {
            follow: crate::instruments_motion::Follow::here(),
            glow: ChartGlow::Theme,
            values: values.into_iter().collect(),
            cols: cols.max(1),
            w: 320.0,
            live: false,
            caption: None,
        }
    }
    crate::instruments_motion::follow_api!();
    /// Total width in px; the hex size follows.
    pub fn width(mut self, w: f32) -> Self {
        self.w = w;
        self
    }
    /// Pulse the cells.
    pub fn live(mut self, on: bool) -> Self {
        self.live = on;
        self
    }
    /// Caption text.
    pub fn caption(mut self, c: impl Into<SharedString>) -> Self {
        self.caption = Some(c.into());
        self
    }
}

impl RenderOnce for HexMap {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let cols = self.cols;
        let values = crate::instruments_motion::follow_all(window, cx, &self.follow, &self.values);
        let rows = values.len().div_ceil(cols);
        let s = self.w / (3f32.sqrt() * (cols as f32 + 0.5));
        let (gw, gh) = hex_extent(rows.max(1), cols, s);
        let live = self.live;
        let base = t.palette.surface_raised;
        let hot = t.palette.primary;
        let hi = t.palette.danger;
        let edge = scale_alpha(hsla(t.palette.border), 0.8);
        let glow = self.glow.resolve(&t);
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let now = clock();
                const LEVELS: usize = 8;
                let mut fills: [Vec<Vec<(f32, f32)>>; LEVELS] = Default::default();
                let mut halos: Vec<Vec<(f32, f32)>> = Vec::new();
                let mut edges: Vec<Vec<(f32, f32)>> = Vec::new();
                for (i, &v0) in values.iter().enumerate() {
                    let (row, col) = (i / cols, i % cols);
                    let v = if live {
                        (v0 * (0.75 + 0.25 * (now * 1.3 + i as f32 * 0.9).sin())).clamp(0.0, 1.0)
                    } else {
                        v0.clamp(0.0, 1.0)
                    };
                    let (cx_, cy_) = hex_center(row, col, s);
                    let inner = hex_points(cx_, cy_, s * 0.9).to_vec();
                    if v > 0.05 {
                        if v > 0.7 && glow.is_some() {
                            halos.push(hex_points(cx_, cy_, s * 1.05).to_vec());
                        }
                        fills[((v * LEVELS as f32) as usize).min(LEVELS - 1)].push(inner.clone());
                    }
                    edges.push(inner);
                }
                fill_batch(w, b.origin, &halos, scale_alpha(hsla(hi), 0.22));
                for (l, polys) in fills.iter().enumerate() {
                    let v = (l as f32 + 0.5) / LEVELS as f32;
                    fill_batch(
                        w,
                        b.origin,
                        polys,
                        hsla(base.mix(if v > 0.8 { hi } else { hot }, v)),
                    );
                }
                stroke_batch(w, b.origin, &edges, 1.0, edge, true);
                if live {
                    crate::instruments_motion::next_frame(w);
                }
            },
        )
        .absolute()
        .size_full();
        frame(&t, gw, gh, self.caption, cv)
    }
}

// ─────────────────────────────── RingStack ───────────────────────────────

/// Concentric 270° rings, one per `(label, percent)`, with a legend.
#[derive(IntoElement)]
pub struct RingStack {
    glow: ChartGlow,
    items: Vec<(SharedString, f32)>,
    size: f32,
}

impl RingStack {
    /// The glow ("shadow") under the lines: the theme's own, none, or a radius.
    pub fn glow(mut self, g: ChartGlow) -> Self {
        self.glow = g;
        self
    }
    /// Create a new [`RingStack`].
    pub fn new<S: Into<SharedString>>(items: impl IntoIterator<Item = (S, f32)>) -> Self {
        Self {
            glow: ChartGlow::Theme,
            items: items.into_iter().map(|(l, v)| (l.into(), v)).collect(),
            size: 150.0,
        }
    }
    /// Size in px.
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
}

impl RenderOnce for RingStack {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let g = crate::motion::grow(window, cx, "RingStack");
        let size = self.size;
        let n = self.items.len().max(1);
        let thick = (size / 2.0 / (n as f32 + 1.5)).clamp(4.0, 12.0);
        let gap = thick * 0.45;
        let fr: Vec<f32> = self
            .items
            .iter()
            .map(|i| norm(i.1, 0.0, 100.0) * g)
            .collect();
        let cols: Vec<Hsla> = (0..n).map(|i| hsla(t.palette.series_at(i))).collect();
        let (track, glow) = (
            scale_alpha(hsla(t.palette.border), 0.8),
            self.glow.resolve(&t),
        );
        let (fr2, cols2) = (fr.clone(), cols.clone());
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let c = size / 2.0;
                let (s0, span) = SWEEP;
                for (i, f) in fr2.iter().enumerate() {
                    let r = ring_radius(i, c - 2.0, thick, gap);
                    stroke(
                        w,
                        b.origin,
                        &turn_arc(c, c, r, s0, s0 + span),
                        thick,
                        track,
                        false,
                    );
                    if *f > 0.0 {
                        glow_stroke(
                            w,
                            b.origin,
                            &turn_arc(c, c, r, s0, s0 + span * f),
                            thick,
                            cols2[i],
                            glow,
                        );
                    }
                }
            },
        )
        .absolute()
        .size_full();
        let legend = self.items.iter().enumerate().map(|(i, (l, v))| {
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(div().w(px(10.0)).h(px(10.0)).bg(cols[i]))
                .child(
                    div()
                        .text_size(px(t.typography.size_sm))
                        .text_color(hsla(t.palette.text_muted))
                        .font_family(first_family(t.typography.mono))
                        .child(format_label(&t.typography, l)),
                )
                .child(
                    div()
                        .text_size(px(t.typography.size_sm))
                        .text_color(hsla(t.palette.text))
                        .font_family(first_family(t.typography.display))
                        .child(format!("{}%", fmt_num((*v * g).round()))),
                )
        });
        div()
            .flex()
            .items_center()
            .gap(px(t.spacing.lg))
            .child(div().relative().w(px(size)).h(px(size)).child(cv))
            .child(div().flex().flex_col().gap(px(6.0)).children(legend))
    }
}
