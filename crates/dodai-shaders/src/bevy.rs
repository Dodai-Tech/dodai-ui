//! bevy (feature `bevy`): a shader as a UI image node.
//!
//! Add [`ShaderPlugin`] and spawn a [`ShaderNode`]; it fills the node's width, its height
//! follows the spec's aspect.
//!
//! ```ignore
//! app.add_plugins(ShaderPlugin);
//! commands.spawn((ShaderNode::new(Spec::new("neon_grid")), Node { width: percent(100), ..default() }));
//! ```

use crate::player::{Live, Poll, Spec};
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

/// A playing shader on a UI node (adds the [`ImageNode`] itself).
#[derive(Component)]
#[require(Node, ImageNode)]
pub struct ShaderNode {
    live: Live,
}

impl ShaderNode {
    /// Draws once the node has a width.
    pub fn new(spec: Spec) -> Self {
        Self {
            live: Live::new(spec),
        }
    }

    /// What is playing.
    pub fn spec(&self) -> &Spec {
        self.live.spec()
    }

    /// Replace the spec (nothing happens if it is unchanged).
    pub fn set_spec(&mut self, spec: Spec) {
        self.live.set_spec(spec);
    }
}

/// Keeps every [`ShaderNode`]'s image current.
pub struct ShaderPlugin;

impl Plugin for ShaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_shader_nodes);
    }
}

fn update_shader_nodes(
    mut images: ResMut<Assets<Image>>,
    mut nodes: Query<(&mut ShaderNode, &mut ImageNode, &mut Node, &ComputedNode)>,
) {
    for (mut shader, mut image, mut node, computed) in &mut nodes {
        let inv = computed.inverse_scale_factor();
        let width = computed.size().x * inv;
        let scale = if inv > 0.0 { 1.0 / inv } else { 1.0 };
        node.aspect_ratio = Some(shader.live.spec().aspect);
        if let Poll::New(f) = shader.live.tick(width, scale) {
            let img = Image::new(
                Extent3d {
                    width: f.width,
                    height: f.height,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                f.rgba(),
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::default(),
            );
            // Replace in place so animation does not leak textures.
            if images.get(&image.image).is_some() {
                let _ = images.insert(&image.image, img);
            } else {
                image.image = images.add(img);
            }
        }
    }
}
