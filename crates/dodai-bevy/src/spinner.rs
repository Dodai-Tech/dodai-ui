//! Busy indicators: a spinning ring and an indeterminate bar. Both move on the app clock and
//! take their colours from the theme.

use bevy::prelude::*;

use dodai_core::busy::{cell_alpha, cell_rect, cells, Cell};
use dodai_core::SpinnerStyle;

use crate::theme::{color, DodaiTheme};

/// A spinner, drawn in the shape the active design gives it: a ring, bars, orbiting dots or
/// stepping squares. `size` is its width and height in px.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct UiSpinner {
    /// Width and height px.
    pub size: f32,
}

/// The shape a spinner's children were last built for, so a theme change rebuilds them.
#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub(crate) struct SpinnerBuilt(Option<SpinnerStyle>);

/// One bar, dot or square of a spinner.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub(crate) struct SpinnerCell {
    style: SpinnerStyle,
    cell: Cell,
}

/// An indeterminate bar: a track with a segment sliding along it.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct UiIndeterminate;

#[derive(Component)]
pub(crate) struct IndeterminateFill;

pub use dodai_core::busy::{slide_left, spin_angle, SEGMENT, SPIN_RATE};

/// A ring of `size` px.
pub fn spinner(size: f32) -> impl Bundle {
    (
        UiSpinner { size },
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
    let style = theme.look().spinner;
    for (e, sp, mut built, mut node) in &mut spinners {
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
                        SpinnerCell { style, cell: c },
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
    let ring = theme.look().spinner == SpinnerStyle::Ring;
    for (_, mut tf, mut bc) in &mut rings {
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
        let a = cell_alpha(p.style, p.cell.lag, t);
        let want = lit.with_alpha(lit.alpha() * a);
        if bg.0 != want {
            bg.0 = want;
        }
        if matches!(p.style, SpinnerStyle::Bars | SpinnerStyle::Grid) {
            let (x, y, w, h) = cell_rect(p.style, &p.cell, a);
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

    fn app(t: dodai_core::Theme) -> App {
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
            (dodai_core::material::light(), 0),
            (dodai_core::cyberpunk::light(), 8),
            (dodai_core::dodai::light(), 9),
            (dodai_core::neobrutalism::light(), 4),
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
        let mut a = app(dodai_core::dodai::light());
        a.world_mut().spawn(spinner(32.0));
        a.update();
        a.update();
        assert_eq!(cell_count(&mut a), 9);
        a.world_mut()
            .resource_mut::<DodaiTheme>()
            .set_theme(dodai_core::neobrutalism::light());
        a.update();
        a.update();
        assert_eq!(cell_count(&mut a), 4);
    }

    #[test]
    fn a_spinner_despawned_in_the_frame_it_is_built_does_not_panic() {
        let mut a = app(dodai_core::dodai::light());
        let e = a.world_mut().spawn(spinner(32.0)).id();
        a.world_mut().commands().entity(e).despawn();
        a.world_mut().flush();
        a.update();
        a.update();
    }
}
