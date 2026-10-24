//! `UiMaterial`s: chamfered surfaces and the scanline overlay.

use bevy::asset::{uuid_handle, Asset, Assets, Handle};
use bevy::color::LinearRgba;
use bevy::math::Vec4;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::{Shader, ShaderRef};
use bevy::ui_render::prelude::UiMaterial;

pub(crate) const CHAMFER_SHADER: Handle<Shader> =
    uuid_handle!("6c1f6f0e-7d0b-4d5c-9a53-0b1c0d5e1a01");
pub(crate) const SCANLINE_SHADER: Handle<Shader> =
    uuid_handle!("6c1f6f0e-7d0b-4d5c-9a53-0b1c0d5e1a02");

pub(crate) fn load_shaders(app: &mut App) {
    let mut shaders = app.world_mut().resource_mut::<Assets<Shader>>();
    let _ = shaders.insert(
        CHAMFER_SHADER.id(),
        Shader::from_wesl(include_str!("shaders/chamfer.wesl"), "dodai/chamfer.wesl"),
    );
    let _ = shaders.insert(
        SCANLINE_SHADER.id(),
        Shader::from_wesl(
            include_str!("shaders/scanlines.wesl"),
            "dodai/scanlines.wesl",
        ),
    );
}

/// Cut-corner fill + border, evaluated per pixel from `Shape` tokens.
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone, PartialEq)]
pub struct ChamferMaterial {
    /// Fill colour (linear).
    #[uniform(0)]
    pub fill: LinearRgba,
    /// Border colour (linear).
    #[uniform(1)]
    pub border: LinearRgba,
    /// `x` border width px, `y` chamfer px.
    #[uniform(2)]
    pub params: Vec4,
    /// Cut flags: top-left, top-right, bottom-right, bottom-left.
    #[uniform(3)]
    pub corners: Vec4,
}

impl UiMaterial for ChamferMaterial {
    fn fragment_shader() -> ShaderRef {
        CHAMFER_SHADER.into()
    }
}

/// Horizontal CRT lines.
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone, PartialEq)]
pub struct ScanlineMaterial {
    /// Line colour (linear, alpha = strength).
    #[uniform(0)]
    pub color: LinearRgba,
    /// `x` period px, `y` line coverage `0..1`.
    #[uniform(1)]
    pub params: Vec4,
}

impl UiMaterial for ScanlineMaterial {
    fn fragment_shader() -> ShaderRef {
        SCANLINE_SHADER.into()
    }
}
