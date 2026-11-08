//! Polylines in a fixed box, drawn as thin rotated nodes (bevy UI has no line primitive).

use bevy::ecs::spawn::SpawnWith;
use bevy::prelude::*;

/// One stroked polyline; points are px from the box's top-left.
#[derive(Debug, Clone, PartialEq)]
pub struct PlotLine {
    /// Stroke colour.
    pub color: Color,
    /// Stroke width px.
    pub width: f32,
    /// Points px.
    pub points: Vec<Vec2>,
}

/// Top-left corner, size and rotation (radians) of the node that draws the segment `a → b`
/// `width` px thick. The node is a hair longer than the segment so joints leave no gap.
pub fn segment_box(a: Vec2, b: Vec2, width: f32) -> (Vec2, Vec2, f32) {
    let d = b - a;
    let len = d.length() + width * 0.5;
    let mid = (a + b) / 2.0;
    let dims = Vec2::new(len, width);
    (mid - dims / 2.0, dims, d.y.atan2(d.x))
}

/// A `w`×`h` px box holding `lines`.
pub fn plot(w: f32, h: f32, lines: Vec<PlotLine>) -> impl Bundle {
    (
        Node {
            width: Val::Px(w),
            height: Val::Px(h),
            flex_shrink: 0.0,
            ..default()
        },
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            for l in &lines {
                for pair in l.points.windows(2) {
                    let (at, dims, rot) = segment_box(pair[0], pair[1], l.width);
                    parent.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(at.x),
                            top: Val::Px(at.y),
                            width: Val::Px(dims.x),
                            height: Val::Px(dims.y),
                            ..default()
                        },
                        UiTransform::from_rotation(Rot2::radians(rot)),
                        BackgroundColor(l.color),
                    ));
                }
            }
        })),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_segment_box_is_centred_on_its_segment_and_turned_along_it() {
        let (a, b) = (Vec2::new(10.0, 10.0), Vec2::new(10.0, 50.0));
        let (at, dims, rot) = segment_box(a, b, 2.0);
        assert!((at + dims / 2.0 - Vec2::new(10.0, 30.0)).length() < 1e-4);
        assert!((rot - std::f32::consts::FRAC_PI_2).abs() < 1e-5);
        assert!(dims.x >= 40.0 && dims.y == 2.0);
    }

    #[test]
    fn a_polyline_of_n_points_spawns_n_minus_one_segments() {
        let mut app = App::new();
        let line = PlotLine {
            color: Color::WHITE,
            width: 1.0,
            points: vec![
                Vec2::ZERO,
                Vec2::X * 5.0,
                Vec2::new(5.0, 5.0),
                Vec2::Y * 9.0,
            ],
        };
        let e = app.world_mut().spawn(plot(20.0, 20.0, vec![line])).id();
        let kids = app.world().get::<Children>(e).unwrap().len();
        assert_eq!(kids, 3);
    }
}
