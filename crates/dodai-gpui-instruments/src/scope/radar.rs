//! [`RadarScope`] and its [`Blip`] contacts.

use super::*;
use crate::draw::polar;
use dodai_gpui::paint::{fill_circle, stroke_circle};

/// A contact on a [`RadarScope`]: `bearing` in turns (0 = up, clockwise, `0.25` = east) and
/// `range` as `0..=1` of the radius.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Blip {
    /// Angle in turns clockwise from north (`0.25` is east).
    pub bearing: f32,
    /// Distance from the centre, 0 to 1.
    pub range: f32,
}

impl Blip {
    /// Create a new [`Blip`].
    pub fn new(bearing: f32, range: f32) -> Self {
        Self { bearing, range }
    }
}

/// Sweeping radar scope with range rings and fading contacts.
#[derive(IntoElement)]
pub struct RadarScope {
    follow: dodai_gpui::follow::Follow,
    sweep: Option<f32>,
    glow: ChartGlow,
    blips: Vec<Blip>,
    size: f32,
    live: bool,
    period: f32,
    caption: Option<SharedString>,
}

impl RadarScope {
    /// The glow ("shadow") under the lines: the theme's own, none, or a radius.
    pub fn glow(mut self, g: ChartGlow) -> Self {
        self.glow = g;
        self
    }
    /// Create a new [`RadarScope`]. Contacts that move glide to their new position in a straight
    /// line (also across north), and new ones appear in place.
    #[track_caller]
    pub fn new(blips: impl IntoIterator<Item = Blip>) -> Self {
        Self {
            follow: dodai_gpui::follow::Follow::here(),
            sweep: None,
            glow: ChartGlow::Theme,
            blips: blips.into_iter().collect(),
            size: 220.0,
            live: true,
            period: 4.0,
            caption: None,
        }
    }
    /// Size in px.
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
    /// Rotate the sweep (default on); off freezes it and shows all contacts.
    pub fn live(mut self, on: bool) -> Self {
        self.live = on;
        self
    }
    dodai_gpui::follow_api!();
    /// Points the beam at this bearing (turns, clockwise from north) instead of rotating on the
    /// clock, e.g. an antenna or a tracked heading. All contacts stay lit, and the beam turns the short way to a new value.
    pub fn sweep(mut self, bearing: f32) -> Self {
        self.sweep = Some(bearing);
        self
    }
    /// Seconds per revolution (default 4).
    pub fn period(mut self, s: f32) -> Self {
        self.period = s.max(0.5);
        self
    }
    /// Caption text.
    pub fn caption(mut self, c: impl Into<SharedString>) -> Self {
        self.caption = Some(c.into());
        self
    }
}

impl RenderOnce for RadarScope {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let (size, live, period) = (self.size, self.live, self.period);
        // Ease each contact as a point so a move across north does not swing the long way round.
        let flat: Vec<f32> = self
            .blips
            .iter()
            .flat_map(|b| {
                let (s, c) = (b.bearing * std::f32::consts::TAU).sin_cos();
                [b.range * s, b.range * c]
            })
            .collect();
        let eased = dodai_gpui::follow::follow_all(window, cx, &self.follow, &flat);
        let blips: Vec<Blip> = eased
            .chunks_exact(2)
            .map(|p| {
                Blip::new(
                    p[0].atan2(p[1]).rem_euclid(std::f32::consts::TAU) / std::f32::consts::TAU,
                    p[0].hypot(p[1]),
                )
            })
            .collect();
        let pointed = self.sweep.map(|b| {
            dodai_gpui::follow::follow_angle(
                window,
                cx,
                &self.follow.sub("sweep"),
                0,
                b.rem_euclid(1.0) * 360.0,
            ) / 360.0
        });
        let live = live && pointed.is_none();
        let col = hsla(t.palette.primary);
        let warn = hsla(t.palette.danger);
        let ring = grid_color(&t);
        let glow = self.glow.resolve(&t);
        let ring_w = look(&t).ring;
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let o = b.origin;
                let c = size / 2.0;
                let r = c - 3.0;
                let rings: Vec<_> = (1..=4)
                    .map(|i| turn_arc(c, c, r * i as f32 / 4.0, 0.0, 1.0))
                    .collect();
                stroke_batch(w, o, &rings, ring_w, ring, true);
                let cross: Vec<_> = (0..4)
                    .map(|q| vec![(c, c), polar(c, c, r, q as f32 / 4.0)])
                    .collect();
                stroke_batch(w, o, &cross, ring_w, ring, false);
                let (mut long, mut short) = (Vec::new(), Vec::new());
                for i in 0..36 {
                    let a = i as f32 / 36.0;
                    let major = i % 3 == 0;
                    let tick = vec![
                        polar(c, c, r, a),
                        polar(c, c, r - if major { 7.0 } else { 3.5 }, a),
                    ];
                    if major {
                        long.push(tick)
                    } else {
                        short.push(tick)
                    }
                }
                stroke_batch(w, o, &long, 1.0, scale_alpha(col, 0.55), false);
                stroke_batch(w, o, &short, 1.0, scale_alpha(col, 0.55), false);
                let sweep = match pointed {
                    Some(a) => a,
                    None if live => (clock() / period).fract(),
                    None => 0.12,
                };
                // fading fan behind the beam: 7 alpha bands of 4 wedges each
                let (bands, per) = (7, 4);
                for band in 0..bands {
                    let tris: Vec<Vec<(f32, f32)>> = (0..per)
                        .map(|j| {
                            let i = band * per + j;
                            let a0 = sweep - 0.22 * (i + 1) as f32 / (bands * per) as f32;
                            let a1 = sweep - 0.22 * i as f32 / (bands * per) as f32;
                            vec![(c, c), polar(c, c, r, a0), polar(c, c, r, a1)]
                        })
                        .collect();
                    let k = 1.0 - (band as f32 + 0.5) / bands as f32;
                    fill_batch(w, o, &tris, scale_alpha(col, 0.28 * k * k));
                }
                glow_stroke(w, o, &[(c, c), polar(c, c, r, sweep)], 2.0, col, glow);
                for bl in &blips {
                    let a = if live {
                        blip_alpha(sweep, bl.bearing)
                    } else {
                        1.0
                    };
                    if a < 0.04 {
                        continue;
                    }
                    let p = polar(c, c, r * bl.range.clamp(0.0, 1.0), bl.bearing);
                    fill_circle(w, o, (p.0, p.1), 4.0, scale_alpha(warn, a));
                    stroke_circle(
                        w,
                        o,
                        (p.0, p.1),
                        7.0 + 5.0 * (1.0 - a),
                        1.0,
                        scale_alpha(warn, a * 0.7),
                    );
                }
                fill_circle(w, o, (c, c), 2.5, col);
                if live {
                    dodai_gpui::follow::next_frame(w);
                }
            },
        )
        .absolute()
        .size_full();
        frame(&t, size, size, self.caption, cv)
    }
}
