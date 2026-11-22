//! Busy indicators: a spinning ring and an indeterminate bar. Both move on the app clock and
//! take their colours from the theme.

use bevy::prelude::*;

use dodai_ui_core::busy::{cells, moves, pose, Cell};
use dodai_ui_core::SpinnerStyle;

use crate::theme::{color, DodaiTheme};

/// A spinner, drawn in the shape the active design gives it: a ring, bars, orbiting dots or
/// stepping squares. `size` is its width and height in px.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct UiSpinner {
    /// Width and height px.
    pub size: f32,
    /// A style of its own; the design's spinner when `None`.
    pub style: Option<SpinnerStyle>,
}

/// The shape a spinner's children were last built for, so a theme change rebuilds them.
#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub(crate) struct SpinnerBuilt(Option<SpinnerStyle>);

/// One bar, dot or square of a spinner.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub(crate) struct SpinnerCell {
    style: SpinnerStyle,
    cell: Cell,
    size: f32,
}

/// An indeterminate bar: a track with a segment sliding along it.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct UiIndeterminate;

#[derive(Component)]
pub(crate) struct IndeterminateFill;

pub use dodai_ui_core::busy::{slide_left, spin_angle, SEGMENT, SPIN_RATE};

/// A ring of `size` px.
pub fn spinner(size: f32) -> impl Bundle {
    spinner_with(size, None)
}

/// A spinner of `size` px in `style`.
pub fn spinner_of(size: f32, style: SpinnerStyle) -> impl Bundle {
    spinner_with(size, Some(style))
}

fn spinner_with(size: f32, style: Option<SpinnerStyle>) -> impl Bundle {
    (
        UiSpinner { size, style },
        SpinnerBuilt::default(),
        Node {
            width: Val::Px(size),
            height: Val::Px(size),
            flex_shrink: 0.0,
            ..default()
        },
        BorderColor::default(),
    )
}

/// An indeterminate bar, full width.
pub fn indeterminate() -> impl Bundle {
    (
        UiIndeterminate,
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(4.0),
            overflow: Overflow::clip(),
            border_radius: BorderRadius::MAX,
            ..default()
        },
        BackgroundColor::default(),
        children![(
            IndeterminateFill,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(SEGMENT * 100.0),
                height: Val::Percent(100.0),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BackgroundColor::default(),
        )],
    )
}

/// Makes a spinner's node and children match the design's spinner shape. Runs when the shape
/// changed since the spinner was built (a new spinner or a theme switch).
pub(crate) fn build_spinners(
    theme: Res<DodaiTheme>,
    mut commands: Commands,
    mut spinners: Query<(Entity, &UiSpinner, &mut SpinnerBuilt, &mut Node)>,
) {
    for (e, sp, mut built, mut node) in &mut spinners {
        let style = sp.style.unwrap_or(theme.look().spinner);
        if built.0 == Some(style) {
            continue;
        }
        built.0 = Some(style);
        let ring = style == SpinnerStyle::Ring;
        node.border = if ring {
            UiRect::all(Val::Px((sp.size / 8.0).max(2.0)))
        } else {
            UiRect::ZERO
        };
        node.border_radius = if ring {
            BorderRadius::MAX
        } else {
            BorderRadius::ZERO
        };
        let list = cells(style, sp.size);
        let size = sp.size;
        // The page may rebuild (and despawn this spinner) in the same frame: skip if it's gone.
        commands.queue(move |world: &mut World| {
            let Ok(mut spinner) = world.get_entity_mut(e) else {
                return;
            };
            spinner.despawn_related::<Children>();
            spinner.with_children(|p| {
                for c in list {
                    let (w, h) = (c.w, c.h);
                    p.spawn((
                        SpinnerCell {
                            style,
                            cell: c,
                            size,
                        },
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(c.x),
                            top: Val::Px(c.y),
                            width: Val::Px(w),
                            height: Val::Px(h),
                            border_radius: if c.round {
                                BorderRadius::MAX
                            } else {
                                BorderRadius::all(Val::Px(c.radius))
                            },
                            ..default()
                        },
                        BackgroundColor::default(),
                    ));
                }
            });
        });
    }
}

pub(crate) fn animate_spinners(
    theme: Res<DodaiTheme>,
    time: Res<Time>,
    mut rings: Query<(&UiSpinner, &mut UiTransform, &mut BorderColor)>,
    mut parts: Query<(&SpinnerCell, &mut Node, &mut BackgroundColor)>,
) {
    let t = time.elapsed_secs();
    let lit = color(theme.palette.primary);
    let dim = color(theme.palette.border);
    for (sp, mut tf, mut bc) in &mut rings {
        let ring = sp.style.unwrap_or(theme.look().spinner) == SpinnerStyle::Ring;
        let (rot, want) = if ring {
            (
                Rot2::radians(spin_angle(t)),
                BorderColor {
                    top: lit,
                    right: dim,
                    bottom: dim,
                    left: dim,
                },
            )
        } else {
            (Rot2::IDENTITY, BorderColor::all(Color::NONE))
        };
        if tf.rotation != rot {
            tf.rotation = rot;
        }
        if *bc != want {
            *bc = want;
        }
    }
    for (p, mut n, mut bg) in &mut parts {
        let (x, y, w, h, a) = pose(p.style, &p.cell, p.size, t);
        let want = lit.with_alpha(lit.alpha() * a);
        if bg.0 != want {
            bg.0 = want;
        }
        if moves(p.style) {
            let (left, top, width, height) = (Val::Px(x), Val::Px(y), Val::Px(w), Val::Px(h));
            if n.left != left || n.top != top || n.width != width || n.height != height {
                n.left = left;
                n.top = top;
                n.width = width;
                n.height = height;
            }
        }
    }
}

/// A progress bar in a design's own style, rebuilt every frame from
/// [`dodai_ui_core::busy::bar_pieces`]. Bevy's UI has no polygons: slanted cells and stripes are
/// drawn upright.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct UiStyledBar {
    /// The style.
    pub style: dodai_ui_core::ProgressStyle,
    /// Fraction 0..=1, or `None` for an indeterminate bar.
    pub value: Option<f32>,
}

/// A styled progress bar, full width.
pub fn styled_progress(style: dodai_ui_core::ProgressStyle, value: Option<f32>) -> impl Bundle {
    (
        UiStyledBar { style, value },
        Node {
            width: Val::Percent(100.0),
            max_width: Val::Px(420.0),
            height: Val::Px(dodai_ui_core::busy::bar_height(style)),
            ..default()
        },
    )
}

pub(crate) fn animate_styled_bars(
    theme: Res<DodaiTheme>,
    time: Res<Time>,
    mut commands: Commands,
    bars: Query<(Entity, &UiStyledBar, &ComputedNode, Option<&Children>)>,
    mut kids: Query<(&mut Node, &mut BackgroundColor), Without<UiStyledBar>>,
) {
    use dodai_ui_core::busy::{bar_pieces, BarRole};
    let t = time.elapsed_secs();
    let k = theme.progress(None);
    let (track, fill, line) = (color(k.track), color(k.fill), color(k.border));
    for (e, bar, node, children) in &bars {
        let w = node.size().x * node.inverse_scale_factor();
        if w <= 0.0 {
            continue;
        }
        let pieces = bar_pieces(bar.style, bar.value, t, w);
        let have = children.map_or(0, |c| c.len());
        // The pieces are updated in place (no respawn, so no flicker); spare ones are hidden.
        for i in 0..have {
            let Some(&kid) = children.and_then(|c| c.get(i)) else {
                continue;
            };
            let Ok((mut n, mut bg)) = kids.get_mut(kid) else {
                continue;
            };
            let Some(pc) = pieces.get(i) else {
                n.display = Display::None;
                continue;
            };
            let xs = pc.pts.map(|q| q.0);
            let ys = pc.pts.map(|q| q.1);
            let (x0, x1) = (
                xs.iter().cloned().fold(f32::MAX, f32::min),
                xs.iter().cloned().fold(f32::MIN, f32::max),
            );
            let (y0, y1) = (
                ys.iter().cloned().fold(f32::MAX, f32::min),
                ys.iter().cloned().fold(f32::MIN, f32::max),
            );
            // A slanted piece is drawn upright, as wide as its top edge.
            let top = (pc.pts[1].0 - pc.pts[0].0).abs();
            let slanted = pc.pts[0].0 != pc.pts[3].0;
            let (x, width) = if slanted && top > 0.0 {
                ((x0 + x1) / 2.0 - top / 2.0, top)
            } else {
                (x0, x1 - x0)
            };
            let base = match pc.role {
                BarRole::Track => track,
                BarRole::Fill => fill,
                BarRole::Line => line,
            };
            n.display = Display::Flex;
            n.left = Val::Px(x);
            n.top = Val::Px(y0);
            n.width = Val::Px(width);
            n.height = Val::Px(y1 - y0);
            n.border_radius = BorderRadius::all(Val::Px(pc.radius));
            bg.0 = base.with_alpha(base.alpha() * pc.alpha);
        }
        if pieces.len() > have {
            let extra = pieces.len() - have;
            commands.entity(e).with_children(|p| {
                for _ in 0..extra {
                    p.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            display: Display::None,
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ));
                }
            });
        }
    }
}

/// A Material 3 wavy bar or circular progress indicator, drawn as a trail of dots along the path.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct UiWavy {
    /// Fraction 0..=1, or `None` for indeterminate.
    pub value: Option<f32>,
    /// A ring instead of a bar.
    pub ring: bool,
    /// Wavy rather than straight.
    pub wavy: bool,
}

/// A wavy linear progress bar.
pub fn wavy_progress(value: Option<f32>) -> impl Bundle {
    (
        UiWavy {
            value,
            ring: false,
            wavy: true,
        },
        Node {
            width: Val::Percent(100.0),
            max_width: Val::Px(420.0),
            height: Val::Px(14.0),
            ..default()
        },
    )
}

/// A circular progress indicator, optionally wavy.
pub fn circular_progress(value: Option<f32>, wavy: bool) -> impl Bundle {
    (
        UiWavy {
            value,
            ring: true,
            wavy,
        },
        Node {
            width: Val::Px(48.0),
            height: Val::Px(48.0),
            ..default()
        },
    )
}

pub(crate) fn animate_wavy(
    theme: Res<DodaiTheme>,
    time: Res<Time>,
    mut commands: Commands,
    bars: Query<(Entity, &UiWavy, &ComputedNode, Option<&Children>)>,
    mut kids: Query<(&mut Node, &mut BackgroundColor), Without<UiWavy>>,
) {
    use dodai_ui_core::busy::{progress_window, wave_line, wave_ring};
    let t = time.elapsed_secs();
    let k = theme.progress(None);
    let (track, fill) = (color(k.track), color(k.fill));
    for (e, w, node, children) in &bars {
        let size = node.size() * node.inverse_scale_factor();
        if size.x <= 0.0 {
            continue;
        }
        let pts = if w.ring {
            let (amp, lobes) = if w.wavy { (2.0, 10) } else { (0.0, 1) };
            wave_ring(size.x.min(size.y), amp, lobes, 120)
        } else {
            let amp = if w.wavy { 3.0 } else { 0.0 };
            wave_line(size.x - 8.0, amp, 24.0, 3.0)
                .into_iter()
                .map(|(x, y)| (x + 4.0, y + size.y / 2.0))
                .collect()
        };
        let win = progress_window(w.value, t, w.ring);
        let have = children.map_or(0, |c| c.len());
        let n = pts.len();
        for (i, &(x, y)) in pts.iter().enumerate().take(have) {
            let Some(&kid) = children.and_then(|c| c.get(i)) else {
                continue;
            };
            let Ok((mut nd, mut bg)) = kids.get_mut(kid) else {
                continue;
            };
            let f = i as f32 / (n - 1) as f32;
            let lit = win.iter().any(|&(a, b)| b > a && f >= a && f <= b);
            let d = 4.0;
            nd.left = Val::Px(x - d / 2.0);
            nd.top = Val::Px(y - d / 2.0);
            bg.0 = if lit {
                fill
            } else {
                track.with_alpha(track.alpha() * 0.3)
            };
        }
        if n > have {
            commands.entity(e).with_children(|p| {
                for _ in have..n {
                    p.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            width: Val::Px(4.0),
                            height: Val::Px(4.0),
                            border_radius: BorderRadius::MAX,
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ));
                }
            });
        }
    }
}

type SegmentFilter = (With<IndeterminateFill>, Without<UiIndeterminate>);

pub(crate) fn animate_bars(
    theme: Res<DodaiTheme>,
    time: Res<Time>,
    mut tracks: Query<(&mut BackgroundColor, &Children), With<UiIndeterminate>>,
    mut fills: Query<(&mut Node, &mut BackgroundColor), SegmentFilter>,
) {
    let t = time.elapsed_secs();
    let track = color(theme.progress(None).track);
    let fill = color(theme.progress(None).fill);
    for (mut bg, kids) in &mut tracks {
        if bg.0 != track {
            bg.0 = track;
        }
        for k in kids.iter() {
            if let Ok((mut n, mut f)) = fills.get_mut(k) {
                n.left = Val::Percent(slide_left(t) * 100.0);
                if f.0 != fill {
                    f.0 = fill;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app(t: dodai_ui_core::Theme) -> App {
        let mut app = App::new();
        let mut theme = DodaiTheme::new();
        theme.set_theme(t);
        app.insert_resource(theme)
            .init_resource::<Time>()
            .add_systems(Update, (build_spinners, animate_spinners).chain());
        app
    }

    fn cell_count(app: &mut App) -> usize {
        let w = app.world_mut();
        w.query::<&SpinnerCell>().iter(w).count()
    }

    #[test]
    fn each_design_builds_its_own_cells() {
        for (t, n) in [
            (dodai_ui_core::material::light(), 20),
            (dodai_ui_core::cyberpunk::light(), 4),
            (dodai_ui_core::dodai::light(), 4),
            (dodai_ui_core::neobrutalism::light(), 3),
        ] {
            let id = t.id();
            let mut a = app(t);
            a.world_mut().spawn(spinner(32.0));
            a.update();
            a.update();
            assert_eq!(cell_count(&mut a), n, "{id}");
        }
    }

    #[test]
    fn a_theme_switch_rebuilds_the_cells_once() {
        let mut a = app(dodai_ui_core::dodai::light());
        a.world_mut().spawn(spinner(32.0));
        a.update();
        a.update();
        assert_eq!(cell_count(&mut a), 4);
        a.world_mut()
            .resource_mut::<DodaiTheme>()
            .set_theme(dodai_ui_core::neobrutalism::light());
        a.update();
        a.update();
        assert_eq!(cell_count(&mut a), 3);
    }

    #[test]
    fn a_spinner_despawned_in_the_frame_it_is_built_does_not_panic() {
        let mut a = app(dodai_ui_core::dodai::light());
        let e = a.world_mut().spawn(spinner(32.0)).id();
        a.world_mut().commands().entity(e).despawn();
        a.world_mut().flush();
        a.update();
        a.update();
    }
}
