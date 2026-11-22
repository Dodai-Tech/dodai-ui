//! The busy shimmer: a light band sweeps across a button, inside its outline
//! ([`dodai_ui_core::motion::button_shimmer`]). Chamfered buttons draw it in their material; native
//! ones get a gradient child with the button's corner radii, laid under the label.

use bevy::prelude::*;
use dodai_ui_core::motion::{button_shimmer, shimmer_stops, BUTTON_SHIMMER_WIDTH};
use dodai_ui_core::ButtonVariant;

use crate::material::ChamferMaterial;
use crate::theme::{color, DodaiTheme};

/// Sweep the busy shimmer over this button; the variant picks its colour
/// ([`dodai_ui_core::Theme::shimmer_colour`]).
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shimmer(pub ButtonVariant);

/// The gradient child that draws the band on a native (non-chamfered) button.
#[derive(Component)]
pub struct ShimmerBand;

/// Move every band to where the sweep is now.
#[allow(clippy::type_complexity)]
pub(crate) fn animate_shimmer(
    time: Res<Time>,
    theme: Res<DodaiTheme>,
    mut materials: ResMut<Assets<ChamferMaterial>>,
    mut commands: Commands,
    buttons: Query<(
        Entity,
        &Shimmer,
        &Node,
        Option<&MaterialNode<ChamferMaterial>>,
        Option<&Children>,
    )>,
    mut bands: Query<(&mut Node, &mut BackgroundGradient), (With<ShimmerBand>, Without<Shimmer>)>,
) {
    let x = button_shimmer(time.elapsed_secs());
    for (e, sh, node, mat, kids) in &buttons {
        let c = theme.theme().shimmer_colour(sh.0);
        let band = kids.and_then(|k| k.iter().find(|k| bands.contains(*k)));
        if let Some(mut m) = mat.and_then(|h| materials.get_mut(&h.0)) {
            m.sheen = color(c).to_linear();
            m.shade.z = x;
            m.shade.w = BUTTON_SHIMMER_WIDTH;
            if let Some((mut n, _)) = band.and_then(|b| bands.get_mut(b).ok()) {
                n.display = Display::None;
            }
            continue;
        }
        let stops: Vec<ColorStop> = shimmer_stops(x)
            .into_iter()
            .map(|(at, a)| {
                ColorStop::new(
                    color(c.with_alpha(a * c.a as f32 / 255.0)),
                    Val::Percent(at * 100.0),
                )
            })
            .collect();
        let gradient = if stops.is_empty() {
            BackgroundGradient(Vec::new())
        } else {
            BackgroundGradient(vec![Gradient::Linear(LinearGradient::to_right(stops))])
        };
        match band.and_then(|b| bands.get_mut(b).ok()) {
            Some((mut n, mut g)) => {
                n.display = Display::Flex;
                if n.border_radius != node.border_radius {
                    n.border_radius = node.border_radius;
                }
                *g = gradient;
            }
            None => {
                let b = commands
                    .spawn((
                        ShimmerBand,
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(0.0),
                            right: Val::Px(0.0),
                            top: Val::Px(0.0),
                            bottom: Val::Px(0.0),
                            border_radius: node.border_radius,
                            ..default()
                        },
                        gradient,
                        bevy::picking::Pickable::IGNORE,
                    ))
                    .id();
                // First child: over the fill, under the label.
                commands.entity(e).insert_children(0, &[b]);
            }
        }
    }
}
