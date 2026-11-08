//! Seven-segment figures: a [`SegmentDisplay`] for numbers and clocks. Unlit segments stay
//! faintly visible like a real display.

use crate::segments;
use dodai_gpui::paint::{fill, scale_alpha};
use dodai_gpui::{hsla, theme_of};
use gpui::{
    canvas, div, px, App, Bounds, Hsla, IntoElement, ParentElement, Pixels, RenderOnce,
    SharedString, Styled, Window,
};

/// A row of seven-segment digits. `:` and `.` are drawn as dots, other characters as blanks.
#[derive(IntoElement)]
pub struct SegmentDisplay {
    follow: dodai_gpui::follow::Follow,
    fade: bool,
    text: SharedString,
    height: f32,
    color: Option<Hsla>,
    ghost: bool,
}

impl SegmentDisplay {
    dodai_gpui::follow_api!();
    /// Segments cross-fade when the text changes instead of switching. Displays built at one
    /// call site are told apart by their order in the frame.
    pub fn fade(mut self, on: bool) -> Self {
        self.fade = on;
        self
    }
    /// Shows `text` (digits, `-`, `:`, `.` and spaces).
    #[track_caller]
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            follow: dodai_gpui::follow::Follow::here(),
            fade: false,
            text: text.into(),
            height: 48.0,
            color: None,
            ghost: true,
        }
    }
    /// Digit height in px.
    pub fn height(mut self, h: f32) -> Self {
        self.height = h.max(10.0);
        self
    }
    /// Lit colour (default: the theme's primary).
    pub fn color(mut self, c: Hsla) -> Self {
        self.color = Some(c);
        self
    }
    /// Whether unlit segments stay faintly visible (default on).
    pub fn ghost(mut self, on: bool) -> Self {
        self.ghost = on;
        self
    }
}

impl RenderOnce for SegmentDisplay {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let h = self.height;
        let on = self.color.unwrap_or_else(|| hsla(t.palette.primary));
        let ghost = if t.is_dark() { 0.09 } else { 0.12 };
        let off = scale_alpha(on, if self.ghost { ghost } else { 0.0 });
        let chars: Vec<char> = self.text.chars().collect();
        let width: f32 = chars.iter().map(|c| segments::advance(*c, h)).sum();
        // Lit amount of every segment, eased so a changed digit cross-fades.
        let flags: Vec<f32> = chars
            .iter()
            .flat_map(|c| (0..7).map(move |i| f32::from(segments::mask(*c) >> i & 1 == 1)))
            .collect();
        let flags = if self.fade {
            dodai_gpui::follow::follow_all(window, cx, &self.follow, &flags)
        } else {
            flags
        };
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let o = b.origin;
                let th = segments::thickness(h);
                let mut x0 = 0.0;
                for (ci, c) in chars.iter().enumerate() {
                    match c {
                        ':' | '.' => {
                            let s = th * 0.9;
                            let xs = x0 + h * 0.08;
                            let ys: &[f32] = if *c == ':' {
                                &[h * 0.3, h * 0.7]
                            } else {
                                &[h - s]
                            };
                            for y in ys {
                                let sq = [(xs, *y), (xs + s, *y), (xs + s, *y + s), (xs, *y + s)]
                                    .map(|p| segments::slant(p, 0.0, h));
                                fill(w, o, &sq, on);
                            }
                        }
                        _ => {
                            for (i, seg) in segments::digit_segments(h).iter().enumerate() {
                                let pts: Vec<(f32, f32)> =
                                    seg.iter().map(|p| segments::slant(*p, x0, h)).collect();
                                let k = flags
                                    .get(ci * 7 + i)
                                    .copied()
                                    .unwrap_or(0.0)
                                    .clamp(0.0, 1.0);
                                fill(w, o, &pts, dodai_gpui::motion::mix(off, on, k));
                            }
                        }
                    }
                    x0 += segments::advance(*c, h);
                }
            },
        )
        .absolute()
        .size_full();
        div().relative().w(px(width + h * 0.1)).h(px(h)).child(cv)
    }
}
