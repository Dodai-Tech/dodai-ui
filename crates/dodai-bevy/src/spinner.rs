//! Busy indicators: a spinning ring and an indeterminate bar. Both move on the app clock and
//! take their colours from the theme.

use bevy::prelude::*;

use crate::theme::{color, DodaiTheme};

/// A spinning ring.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct UiSpinner;

/// An indeterminate bar: a track with a segment sliding along it.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct UiIndeterminate;

#[derive(Component)]
pub(crate) struct IndeterminateFill;

/// Revolutions per second of the ring.
pub const SPIN_RATE: f32 = 0.9;
/// Width of the sliding segment as a fraction of the track.
pub const SEGMENT: f32 = 0.3;

/// Angle (radians) of the ring after `t` seconds.
pub fn spin_angle(t: f32) -> f32 {
    (t * SPIN_RATE).fract() * std::f32::consts::TAU
}

/// Left edge of the sliding segment as a fraction of the track after `t` seconds. It sweeps from
/// just off the left to just off the right, so the loop never jumps.
pub fn slide_left(t: f32) -> f32 {
    let p = (t * 0.7).fract();
    -SEGMENT + p * (1.0 + SEGMENT)
}

/// A ring of `size` px.
pub fn spinner(size: f32) -> impl Bundle {
    (
        UiSpinner,
        Node {
            width: Val::Px(size),
            height: Val::Px(size),
            border: UiRect::all(Val::Px((size / 8.0).max(2.0))),
            border_radius: BorderRadius::MAX,
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

pub(crate) fn animate_spinners(
    theme: Res<DodaiTheme>,
    time: Res<Time>,
    mut rings: Query<(&mut UiTransform, &mut BorderColor), With<UiSpinner>>,
) {
    let t = time.elapsed_secs();
    let lit = color(theme.palette.primary);
    let dim = color(theme.palette.border);
    for (mut tf, mut bc) in &mut rings {
        tf.rotation = Rot2::radians(spin_angle(t));
        let want = BorderColor {
            top: lit,
            right: dim,
            bottom: dim,
            left: dim,
        };
        if *bc != want {
            *bc = want;
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
