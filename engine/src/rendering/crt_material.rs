//! CRT post-processing material for the display quad.
//!
//! Applies scanlines, barrel distortion, color bleeding, and vignette
//! to the offscreen render texture when CRT mode is enabled.

use bevy::asset::embedded_asset;
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use bevy::sprite_render::{Material2d, Material2dPlugin};

pub struct CrtMaterialPlugin;

impl Plugin for CrtMaterialPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "crt.wgsl");
        app.add_plugins(Material2dPlugin::<CrtMaterial>::default());
    }
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct CrtMaterial {
    #[uniform(0)]
    pub params: CrtParams,
    #[texture(1)]
    #[sampler(2)]
    pub texture: Option<Handle<Image>>,
}

#[derive(Reflect, Debug, Clone, Copy, Default, ShaderType)]
pub struct CrtParams {
    pub scanline_intensity: f32,
    pub barrel_distortion: f32,
    pub color_bleeding: f32,
    pub enabled: u32,
}

impl Material2d for CrtMaterial {
    fn fragment_shader() -> ShaderRef {
        "embedded://dj_engine/rendering/crt.wgsl".into()
    }
}
