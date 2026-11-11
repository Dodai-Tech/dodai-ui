//! Resolved styles of the small token-styled boxes every toolkit draws the same way: cards,
//! alerts, chips, avatars, code, tooltips, rows, steps, skeletons, popups and the closed dropdown.

use crate::{CardVariant, Color, ItemState, StepState, SurfaceTokens, Theme, Tone};

/// What a token-styled box is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kit {
    /// A card.
    Card(CardVariant),
    /// A message bar.
    Alert {
        /// Status tone.
        tone: Tone,
        /// Solid full-width banner.
        banner: bool,
    },
    /// A snackbar.
    Snackbar,
    /// A filter chip.
    Chip {
        /// Selected.
        selected: bool,
    },
    /// A round avatar.
    Avatar,
    /// A code listing.
    Code,
    /// A tooltip bubble.
    Tooltip {
        /// Title-and-body variant.
        rich: bool,
    },
    /// A list, menu or tree row.
    Row {
        /// Selected.
        selected: bool,
    },
    /// A number bubble of a stepper.
    StepDot(StepState),
    /// A small status dot (timeline marker, badge dot).
    Dot(Option<Tone>),
    /// A count badge.
    Count(Tone),
    /// A pagination item.
    Page {
        /// The current page.
        selected: bool,
    },
    /// A loading placeholder.
    Skeleton,
    /// A floating menu or dropdown list.
    Popup,
    /// The closed dropdown button.
    Select,
    /// Header row of a table.
    TableHead,
    /// Body row of a table (`true` = banded).
    TableRow(bool),
    /// A bar of actions: docked, or a floating (lifted, pill on tonal designs) bar; `vibrant`
    /// takes the primary container colours on tonal designs.
    Toolbar {
        /// Lifted and hugging its content.
        floating: bool,
        /// Primary container colours.
        vibrant: bool,
    },
    /// A floating action button (`extended` = with a label).
    Fab {
        /// Carries a label next to the icon.
        extended: bool,
    },
    /// A raised button: the design's elevated button colours, lifted.
    Elevated,
    /// The strip of a segmented control: one bordered box around its segments (a pill
    /// outlined in `outline` on tonal designs).
    Segmented,
    /// One option of a segmented control; the selected one is filled.
    Segment {
        /// The picked option.
        selected: bool,
    },
}

impl Kit {
    /// Which kind of surface the box is for the surface mix and glass (`None`: it only takes a
    /// component's own [`Surface`](crate::Surface)).
    pub fn surface_class(self) -> Option<crate::SurfaceClass> {
        use crate::SurfaceClass as C;
        match self {
            Self::Card(_) | Self::Popup => Some(C::Card),
            Self::Toolbar { .. } => Some(C::Bar),
            Self::Chip { .. } => Some(C::Tag),
            Self::Fab { .. } | Self::Elevated => Some(C::Button),
            Self::Select => Some(C::Input),
            _ => None,
        }
    }

    /// A stable kebab-case name, unique per variant (`card-elevated`, `alert-danger-banner`,
    /// `chip-on`): what stylesheets key on.
    pub fn name(self) -> String {
        let tone = |t: Tone| match t {
            Tone::Info => "info",
            Tone::Success => "success",
            Tone::Warning => "warning",
            Tone::Danger => "danger",
        };
        let on = |b: bool| if b { "on" } else { "off" };
        match self {
            Kit::Card(CardVariant::Elevated) => "card-elevated".into(),
            Kit::Card(CardVariant::Filled) => "card-filled".into(),
            Kit::Card(CardVariant::Outlined) => "card-outlined".into(),
            Kit::Alert { tone: t, banner } => {
                format!("alert-{}{}", tone(t), if banner { "-banner" } else { "" })
            }
            Kit::Snackbar => "snackbar".into(),
            Kit::Chip { selected } => format!("chip-{}", on(selected)),
            Kit::Avatar => "avatar".into(),
            Kit::Code => "code".into(),
            Kit::Tooltip { rich } => if rich { "tooltip-rich" } else { "tooltip" }.into(),
            Kit::Row { selected } => format!("row-{}", on(selected)),
            Kit::StepDot(StepState::Done) => "step-done".into(),
            Kit::StepDot(StepState::Current) => "step-current".into(),
            Kit::StepDot(StepState::Todo) => "step-todo".into(),
            Kit::Dot(None) => "dot".into(),
            Kit::Dot(Some(t)) => format!("dot-{}", tone(t)),
            Kit::Count(t) => format!("count-{}", tone(t)),
            Kit::Page { selected } => format!("page-{}", on(selected)),
            Kit::Skeleton => "skeleton".into(),
            Kit::Popup => "popup".into(),
            Kit::Select => "select".into(),
            Kit::TableHead => "table-head".into(),
            Kit::TableRow(banded) => if banded {
                "table-row-band"
            } else {
                "table-row"
            }
            .into(),
            Kit::Toolbar { floating, vibrant } => format!(
                "toolbar{}{}",
                if floating { "-floating" } else { "" },
                if vibrant { "-vibrant" } else { "" }
            ),
            Kit::Fab { extended } => if extended { "fab-extended" } else { "fab" }.into(),
            Kit::Elevated => "elevated".into(),
            Kit::Segmented => "segmented".into(),
            Kit::Segment { selected } => format!("segment-{}", on(selected)),
        }
    }

    /// Every variant, for stylesheets that need them all.
    pub fn all() -> Vec<Kit> {
        let tones = [Tone::Info, Tone::Success, Tone::Warning, Tone::Danger];
        let mut v = vec![
            Kit::Card(CardVariant::Elevated),
            Kit::Card(CardVariant::Filled),
            Kit::Card(CardVariant::Outlined),
            Kit::Snackbar,
            Kit::Avatar,
            Kit::Code,
            Kit::Tooltip { rich: false },
            Kit::Tooltip { rich: true },
            Kit::StepDot(StepState::Done),
            Kit::StepDot(StepState::Current),
            Kit::StepDot(StepState::Todo),
            Kit::Dot(None),
            Kit::Skeleton,
            Kit::Popup,
            Kit::Select,
            Kit::TableHead,
            Kit::TableRow(false),
            Kit::TableRow(true),
            Kit::Fab { extended: false },
            Kit::Fab { extended: true },
            Kit::Elevated,
            Kit::Segmented,
        ];
        for floating in [false, true] {
            for vibrant in [false, true] {
                v.push(Kit::Toolbar { floating, vibrant });
            }
        }
        for selected in [false, true] {
            v.extend([
                Kit::Chip { selected },
                Kit::Row { selected },
                Kit::Page { selected },
                Kit::Segment { selected },
            ]);
        }
        for t in tones {
            v.extend([
                Kit::Alert {
                    tone: t,
                    banner: false,
                },
                Kit::Alert {
                    tone: t,
                    banner: true,
                },
                Kit::Dot(Some(t)),
                Kit::Count(t),
            ]);
        }
        v
    }
}

/// How a [`Kit`] looks under a theme: pure data, no ECS.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KitStyle {
    /// Fill (`None` = transparent).
    pub fill: Option<Color>,
    /// Outline.
    pub border: Option<Color>,
    /// Text colour inside.
    pub fg: Color,
    /// Glow halo.
    pub glow: bool,
    /// Fully round on rounded designs.
    pub pill: bool,
    /// Horizontal and vertical padding px.
    pub padding: (f32, f32),
    /// Lifted off the page (the design's elevation shadow or hard shadow).
    pub lift: bool,
}

impl Theme {
    /// The style of `kit` on this theme; `hovered` is the pointer state of clickable kits.
    pub fn kit(&self, kit: Kit, hovered: bool) -> KitStyle {
        let theme = self;
        let p = &theme.palette;
        let sp = &theme.spacing;
        let from = |t: SurfaceTokens, padding: (f32, f32), pill: bool| KitStyle {
            fill: t.background,
            border: t.border,
            fg: t.foreground,
            glow: t.glow,
            pill,
            padding,
            lift: false,
        };
        let state = if hovered {
            crate::ControlState::Hovered
        } else {
            crate::ControlState::Idle
        };
        let button = |b: crate::ButtonTokens, padding: (f32, f32)| KitStyle {
            fill: b.background,
            border: b.border,
            fg: b.foreground,
            glow: false,
            pill: true,
            padding,
            lift: true,
        };
        match kit {
            Kit::Card(v) => KitStyle {
                lift: v == CardVariant::Elevated,
                ..from(theme.card(v), (sp.md, sp.md), false)
            },
            Kit::Alert { tone, banner } => from(theme.alert(tone, banner), (sp.md, sp.sm), false),
            Kit::Snackbar => from(theme.snackbar().0, (sp.md, sp.sm), false),
            Kit::Chip { selected } => from(
                theme.chip(ItemState::of(selected, hovered)),
                (sp.md, sp.xs),
                true,
            ),
            Kit::Avatar => KitStyle {
                fill: Some(p.series[0]),
                border: None,
                fg: theme.on(p.series[0]),
                glow: false,
                pill: true,
                padding: (0.0, 0.0),
                lift: false,
            },
            Kit::Code => KitStyle {
                fill: Some(p.roles.surface_container_highest),
                border: Some(p.border),
                fg: p.text,
                glow: false,
                pill: false,
                padding: (sp.md, sp.md),
                lift: false,
            },
            Kit::Tooltip { rich } => {
                let (bg, fg) = theme.tooltip(rich);
                KitStyle {
                    fill: Some(bg),
                    border: Some(p.border),
                    fg,
                    glow: false,
                    pill: false,
                    padding: (sp.md, sp.xs + 2.0),
                    lift: false,
                }
            }
            Kit::Row { selected } => {
                let t = theme.list_item(ItemState::of(selected, hovered));
                KitStyle {
                    fill: t.background,
                    border: None,
                    fg: t.foreground,
                    glow: false,
                    pill: false,
                    padding: (sp.md, sp.sm),
                    lift: false,
                }
            }
            Kit::StepDot(state) => {
                let t = theme.step(state);
                KitStyle {
                    fill: t.fill,
                    border: Some(t.stroke),
                    fg: t.mark,
                    glow: t.glow,
                    pill: true,
                    padding: (0.0, 0.0),
                    lift: false,
                }
            }
            Kit::Dot(tone) => {
                let c = tone.map_or(p.text_muted, |t| p.tone(t));
                KitStyle {
                    fill: Some(c),
                    border: None,
                    fg: theme.on(c),
                    glow: false,
                    pill: true,
                    padding: (0.0, 0.0),
                    lift: false,
                }
            }
            Kit::Count(tone) => from(theme.badge(tone), (sp.xs + 2.0, 0.0), true),
            Kit::Page { selected } => {
                let t = theme.chip(ItemState::of(selected, hovered));
                KitStyle {
                    fill: t.background,
                    border: if selected { t.border } else { None },
                    fg: t.foreground,
                    glow: false,
                    pill: true,
                    padding: (sp.sm, 0.0),
                    lift: false,
                }
            }
            Kit::Popup => KitStyle {
                lift: true,
                ..from(theme.popup(), (sp.xs, sp.xs), false)
            },
            Kit::Toolbar { floating, vibrant } => {
                let tonal = theme.look().tonal;
                let panel = theme.panel(false);
                let (fill, border, fg) = match (tonal, vibrant) {
                    (true, true) => (
                        p.roles.primary_container,
                        None,
                        p.roles.on_primary_container,
                    ),
                    (true, false) => (p.roles.surface_container, None, p.text),
                    (false, _) => (panel.background, Some(panel.border), p.text),
                };
                KitStyle {
                    fill: Some(fill),
                    border,
                    fg,
                    glow: false,
                    pill: tonal && floating,
                    padding: (sp.sm, sp.xs),
                    lift: tonal && floating,
                }
            }
            Kit::Fab { extended } => button(
                theme.fab(crate::ButtonVariant::Primary, state),
                (if extended { sp.lg } else { sp.md }, sp.md),
            ),
            Kit::Elevated => {
                // As tall as a medium button: the label line plus even padding.
                let m = theme.control_metrics(crate::ControlSize::Medium);
                let py = ((m.height - m.text_size * 1.3) / 2.0).max(0.0);
                button(theme.elevated_button(state), (m.padding, py))
            }
            Kit::Select => {
                let i = theme.input(hovered);
                KitStyle {
                    fill: Some(i.background),
                    border: Some(i.border),
                    fg: i.text,
                    glow: false,
                    pill: false,
                    padding: (sp.md, sp.sm),
                    lift: false,
                }
            }
            Kit::Segmented => {
                let tonal = theme.look().tonal;
                KitStyle {
                    fill: (!tonal).then_some(p.background),
                    border: Some(if tonal {
                        p.roles.outline
                    } else {
                        p.border_strong
                    }),
                    fg: p.text,
                    glow: false,
                    pill: tonal,
                    padding: if tonal {
                        (0.0, 0.0)
                    } else {
                        (sp.half(), sp.half())
                    },
                    lift: false,
                }
            }
            Kit::Segment { selected } => {
                let tonal = theme.look().tonal;
                let (fill, fg) = match (tonal, selected) {
                    (true, true) => (
                        Some(p.roles.secondary_container),
                        p.roles.on_secondary_container,
                    ),
                    (true, false) => (
                        hovered.then(|| p.roles.on_surface.with_alpha(0.08)),
                        p.roles.on_surface,
                    ),
                    (false, true) => (Some(p.primary), p.on_primary),
                    (false, false) => (
                        hovered.then_some(p.surface_raised),
                        if hovered { p.text } else { p.text_muted },
                    ),
                };
                KitStyle {
                    fill,
                    border: None,
                    fg,
                    glow: selected && !tonal,
                    pill: tonal,
                    padding: (if tonal { sp.sm * 1.5 } else { sp.md }, 0.0),
                    lift: false,
                }
            }
            Kit::Skeleton => KitStyle {
                fill: Some(p.surface_raised),
                border: None,
                fg: p.text_muted,
                glow: false,
                pill: false,
                padding: (0.0, 0.0),
                lift: false,
            },
            Kit::TableHead => KitStyle {
                fill: Some(p.surface_raised),
                border: None,
                fg: p.text_muted,
                glow: false,
                pill: false,
                padding: (sp.md, sp.sm),
                lift: false,
            },
            Kit::TableRow(banded) => KitStyle {
                fill: banded.then_some(p.roles.surface_container_low),
                border: None,
                fg: p.text,
                glow: false,
                pill: false,
                padding: (sp.md, sp.sm),
                lift: false,
            },
        }
    }
}

/// Motion of the busy indicators, shared by every toolkit so they all turn at the same rate.
pub mod busy {
    use crate::SpinnerStyle;

    /// Revolutions per second of the ring.
    pub const SPIN_RATE: f32 = 0.9;
    /// Width of the sliding segment as a fraction of the track.
    pub const SEGMENT: f32 = 0.3;

    /// Angle (radians) of the ring after `t` seconds.
    pub fn spin_angle(t: f32) -> f32 {
        (t * SPIN_RATE).fract() * std::f32::consts::TAU
    }

    /// Bars or dots in a [`SpinnerStyle::Bars`] / [`SpinnerStyle::Orbit`] spinner.
    pub const TRAIL: usize = 8;
    /// Opacity of a cell the head has not reached for a while.
    pub const DIM: f32 = 0.15;

    /// One bar, dot or square of a spinner, in a `size`×`size` box.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Cell {
        /// Left px.
        pub x: f32,
        /// Top px.
        pub y: f32,
        /// Width px.
        pub w: f32,
        /// Height px (a bar's full height).
        pub h: f32,
        /// A dot instead of a rectangle.
        pub round: bool,
        /// Corner radius px of a rectangle (zero for hard corners).
        pub radius: f32,
        /// Fraction of the cycle at which the bright head reaches this cell.
        pub lag: f32,
    }

    /// The cells of a spinner `size` px across; none for the ring, which is an arc.
    pub fn cells(style: SpinnerStyle, size: f32) -> Vec<Cell> {
        match style {
            SpinnerStyle::Ring => vec![],
            SpinnerStyle::Bars => {
                let pitch = size / TRAIL as f32;
                let w = pitch * 0.58;
                (0..TRAIL)
                    .map(|i| Cell {
                        x: i as f32 * pitch + (pitch - w) / 2.0,
                        y: 0.0,
                        w,
                        h: size,
                        round: false,
                        radius: 0.0,
                        lag: i as f32 / TRAIL as f32,
                    })
                    .collect()
            }
            SpinnerStyle::Orbit => {
                let d = size * 0.18;
                (0..TRAIL)
                    .map(|i| {
                        let (cx, cy) = orbit_xy(i, TRAIL, size / 2.0, size / 2.0 - d / 2.0);
                        Cell {
                            x: cx - d / 2.0,
                            y: cy - d / 2.0,
                            w: d,
                            h: d,
                            round: true,
                            radius: d / 2.0,
                            lag: i as f32 / TRAIL as f32,
                        }
                    })
                    .collect()
            }
            SpinnerStyle::Steps => {
                let side = size * 0.42;
                let far = size - side;
                [(0.0, 0.0), (far, 0.0), (far, far), (0.0, far)]
                    .into_iter()
                    .enumerate()
                    .map(|(i, (x, y))| Cell {
                        x,
                        y,
                        w: side,
                        h: side,
                        round: false,
                        radius: 0.0,
                        lag: i as f32 / 4.0,
                    })
                    .collect()
            }
            SpinnerStyle::Grid => {
                let side = size * 0.28;
                let gap = (size - 3.0 * side) / 2.0;
                (0..9)
                    .map(|i| {
                        let (col, row) = ((i % 3) as f32, (i / 3) as f32);
                        Cell {
                            x: col * (side + gap),
                            y: row * (side + gap),
                            w: side,
                            h: side,
                            round: false,
                            radius: side * 0.3,
                            lag: (col + row) / 5.0,
                        }
                    })
                    .collect()
            }
        }
    }

    /// Opacity of a cell with `lag` after `t` seconds: a bright head chasing a fading tail, or
    /// for [`SpinnerStyle::Steps`] a hard on/off with no fade.
    pub fn cell_alpha(style: SpinnerStyle, lag: f32, t: f32) -> f32 {
        let since = ((t * SPIN_RATE).fract() - lag).rem_euclid(1.0);
        match style {
            SpinnerStyle::Grid => 1.0 - (since * 2.0).min(1.0) * (1.0 - DIM),
            SpinnerStyle::Steps => {
                if since < 0.25 {
                    1.0
                } else {
                    DIM
                }
            }
            _ => 1.0 - since * (1.0 - DIM),
        }
    }

    /// Where a cell is drawn at opacity `alpha`: a bar of [`SpinnerStyle::Bars`] shrinks towards
    /// its foot, a square of [`SpinnerStyle::Grid`] towards its centre, the rest stay put.
    /// Returns `(x, y, w, h)` px.
    pub fn cell_rect(style: SpinnerStyle, c: &Cell, alpha: f32) -> (f32, f32, f32, f32) {
        match style {
            SpinnerStyle::Bars => {
                let h = c.h * bar_scale(alpha);
                (c.x, c.y + c.h - h, c.w, h)
            }
            SpinnerStyle::Grid => {
                let k = 0.35 + 0.65 * alpha;
                let (w, h) = (c.w * k, c.h * k);
                (c.x + (c.w - w) / 2.0, c.y + (c.h - h) / 2.0, w, h)
            }
            _ => (c.x, c.y, c.w, c.h),
        }
    }

    /// Height of a bar of a [`SpinnerStyle::Bars`] spinner as a fraction of its full height.
    pub fn bar_scale(alpha: f32) -> f32 {
        0.4 + 0.6 * alpha
    }

    /// Centre of dot `i` of `n` on a circle of radius `r` around `(c, c)`, starting at the top
    /// and going clockwise.
    pub fn orbit_xy(i: usize, n: usize, c: f32, r: f32) -> (f32, f32) {
        let a = i as f32 / n.max(1) as f32 * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
        (c + r * a.cos(), c + r * a.sin())
    }

    /// Left edge of the sliding segment as a fraction of the track after `t` seconds. It sweeps from
    /// just off the left to just off the right, so the loop never jumps.
    pub fn slide_left(t: f32) -> f32 {
        let p = (t * 0.7).fract();
        -SEGMENT + p * (1.0 + SEGMENT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::themes;

    fn all_kits() -> Vec<Kit> {
        let mut v = vec![
            Kit::Avatar,
            Kit::Code,
            Kit::Snackbar,
            Kit::Skeleton,
            Kit::Popup,
            Kit::Select,
            Kit::TableHead,
            Kit::TableRow(true),
            Kit::TableRow(false),
            Kit::Dot(None),
        ];
        for sel in [false, true] {
            v.extend([
                Kit::Chip { selected: sel },
                Kit::Row { selected: sel },
                Kit::Page { selected: sel },
            ]);
        }
        for rich in [false, true] {
            v.push(Kit::Tooltip { rich });
        }
        for c in [
            CardVariant::Elevated,
            CardVariant::Filled,
            CardVariant::Outlined,
        ] {
            v.push(Kit::Card(c));
        }
        for tone in [Tone::Info, Tone::Success, Tone::Warning, Tone::Danger] {
            v.extend([
                Kit::Alert { tone, banner: true },
                Kit::Alert {
                    tone,
                    banner: false,
                },
                Kit::Dot(Some(tone)),
                Kit::Count(tone),
            ]);
        }
        for s in [StepState::Done, StepState::Current, StepState::Todo] {
            v.push(Kit::StepDot(s));
        }
        v
    }

    #[test]
    fn every_style_but_the_ring_has_cells_inside_its_box() {
        use super::busy::*;
        use crate::SpinnerStyle::*;
        assert!(cells(Ring, 24.0).is_empty());
        for (st, n) in [(Bars, TRAIL), (Orbit, TRAIL), (Steps, 4), (Grid, 9)] {
            let cs = cells(st, 40.0);
            assert_eq!(cs.len(), n);
            for c in cs {
                assert!(c.x >= -1e-3 && c.y >= -1e-3, "{st:?}");
                assert!(c.x + c.w <= 40.001 && c.y + c.h <= 40.001, "{st:?}");
                assert!((0.0..1.0).contains(&c.lag));
            }
        }
    }

    #[test]
    fn the_head_is_brightest_and_the_tail_fades_behind_it() {
        use super::busy::*;
        use crate::SpinnerStyle::Bars;
        let t = 0.4;
        let a: Vec<f32> = cells(Bars, 40.0)
            .iter()
            .map(|c| cell_alpha(Bars, c.lag, t))
            .collect();
        assert!(a.iter().all(|x| *x >= DIM - 1e-4 && *x <= 1.0));
        let (hi, lo) = (
            a.iter().cloned().fold(0.0, f32::max),
            a.iter().cloned().fold(1.0, f32::min),
        );
        assert!(hi > 0.9 && lo < 0.3, "{a:?}");
        assert!(bar_scale(1.0) == 1.0 && bar_scale(0.0) < bar_scale(1.0));
    }

    #[test]
    fn steps_light_one_square_at_a_time_clockwise() {
        use super::busy::*;
        use crate::SpinnerStyle::Steps;
        let cs = cells(Steps, 40.0);
        let mut seen = vec![];
        for k in 0..60 {
            let t = k as f32 * 0.02;
            let lit: Vec<usize> = (0..4)
                .filter(|i| cell_alpha(Steps, cs[*i].lag, t) == 1.0)
                .collect();
            assert_eq!(lit.len(), 1, "t={t}");
            if seen.last() != Some(&lit[0]) {
                seen.push(lit[0]);
            }
        }
        assert_eq!(&seen[..4], &[0, 1, 2, 3]);
    }

    #[test]
    fn orbit_dots_lie_on_the_circle_starting_at_the_top() {
        use super::busy::*;
        let (x, y) = orbit_xy(0, 8, 10.0, 6.0);
        assert!((x - 10.0).abs() < 1e-4 && (y - 4.0).abs() < 1e-4);
        for i in 0..8 {
            let (x, y) = orbit_xy(i, 8, 10.0, 6.0);
            assert!((((x - 10.0).powi(2) + (y - 10.0).powi(2)).sqrt() - 6.0).abs() < 1e-4);
        }
    }

    #[test]
    fn each_design_names_its_own_spinner() {
        use crate::SpinnerStyle::*;
        assert_eq!(crate::material::light().look().spinner, Ring);
        assert_eq!(crate::cyberpunk::variants()[0].look().spinner, Bars);
        assert_eq!(crate::dodai::light().look().spinner, Grid);
        assert_eq!(crate::neobrutalism::variants()[0].look().spinner, Steps);
    }

    #[test]
    fn every_kit_resolves_on_every_theme_with_finite_padding() {
        for t in themes::all() {
            for k in all_kits() {
                for hover in [false, true] {
                    let s = t.kit(k, hover);
                    assert!(s.padding.0 >= 0.0 && s.padding.1 >= 0.0, "{} {k:?}", t.id());
                }
            }
        }
    }

    #[test]
    fn filled_kits_stay_legible() {
        for t in themes::all() {
            for k in all_kits() {
                let s = t.kit(k, false);
                if let Some(fill) = s.fill {
                    if fill.a == 255 && !matches!(k, Kit::StepDot(_) | Kit::Skeleton) {
                        assert!(
                            fill.contrast(s.fg) >= 2.5,
                            "{} {k:?}: {:?} on {:?}",
                            t.id(),
                            s.fg,
                            fill
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn names_are_unique_kebab_case_and_all_covers_every_variant() {
        let all = Kit::all();
        let mut names: Vec<String> = all.iter().map(|k| k.name()).collect();
        assert!(names
            .iter()
            .all(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_lowercase() || c == '-')));
        let n = names.len();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), n, "duplicate kit names");
        // 3 cards + snackbar, avatar, code, 2 tooltips, 3 steps, dot, skeleton, popup, select,
        // head, 2 rows of the table, 2 fabs, elevated, segmented strip, 4 toolbars,
        // 4 selectable kinds x 2, 4 tones x 4.
        assert_eq!(
            n,
            3 + 3 + 2 + 3 + 1 + 1 + 1 + 1 + 1 + 2 + 2 + 1 + 1 + 4 + 8 + 16
        );
    }

    #[test]
    fn floating_toolbars_and_buttons_lift_and_docked_bars_do_not() {
        for t in themes::all() {
            assert!(t.kit(Kit::Fab { extended: false }, false).lift);
            assert!(t.kit(Kit::Elevated, false).lift);
            let docked = t.kit(
                Kit::Toolbar {
                    floating: false,
                    vibrant: false,
                },
                false,
            );
            assert!(!docked.lift && !docked.pill);
            let floating = t.kit(
                Kit::Toolbar {
                    floating: true,
                    vibrant: false,
                },
                false,
            );
            assert_eq!(floating.lift, t.look().tonal);
        }
    }

    #[test]
    fn selection_changes_chip_and_hover_changes_row() {
        let t = themes::all().remove(0);
        assert_ne!(
            t.kit(Kit::Chip { selected: true }, false),
            t.kit(Kit::Chip { selected: false }, false)
        );
        assert_ne!(
            t.kit(Kit::Row { selected: false }, true),
            t.kit(Kit::Row { selected: false }, false)
        );
    }

    #[test]
    fn elevated_card_differs_from_outlined() {
        for t in themes::all() {
            assert_ne!(
                t.kit(Kit::Card(CardVariant::Elevated), false),
                t.kit(Kit::Card(CardVariant::Outlined), false),
                "{}",
                t.id()
            );
        }
    }
}

#[cfg(test)]
mod busy_tests {
    use super::busy::*;

    #[test]
    fn the_ring_turns_once_per_period_and_stays_in_range() {
        for i in 0..200 {
            let a = spin_angle(i as f32 * 0.037);
            assert!((0.0..std::f32::consts::TAU).contains(&a));
        }
        assert!(spin_angle(1.0 / SPIN_RATE - 1e-3) > 6.0);
    }

    #[test]
    fn the_segment_enters_from_the_left_and_leaves_on_the_right() {
        assert!((slide_left(0.0) + SEGMENT).abs() < 1e-6);
        let end = slide_left(1.0 / 0.7 - 1e-3);
        assert!(end > 0.99, "{end}");
        for i in 0..300 {
            let l = slide_left(i as f32 * 0.013);
            assert!((-SEGMENT..=1.0).contains(&l));
        }
    }
}
