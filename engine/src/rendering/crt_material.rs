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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crt_params_default() {
        let params = CrtParams::default();
        assert_eq!(params.scanline_intensity, 0.0);
        assert_eq!(params.barrel_distortion, 0.0);
        assert_eq!(params.color_bleeding, 0.0);
        assert_eq!(params.enabled, 0);
    }

    #[test]
    fn test_crt_params_custom() {
        let params = CrtParams {
            scanline_intensity: 0.3,
            barrel_distortion: 0.15,
            color_bleeding: 0.05,
            enabled: 1,
        };
        assert_eq!(params.scanline_intensity, 0.3);
        assert_eq!(params.enabled, 1);
    }

    #[test]
    fn test_crt_material_shader_path() {
        let shader = CrtMaterial::fragment_shader();
        match shader {
            ShaderRef::Path(path) => {
                assert!(
                    path.to_string().contains("crt.wgsl"),
                    "Shader path should reference crt.wgsl, got: {}",
                    path
                );
            }
            _ => panic!("Expected ShaderRef::Path"),
        }
    }

    #[test]
    fn test_crt_material_no_texture() {
        let mat = CrtMaterial {
            params: CrtParams::default(),
            texture: None,
        };
        assert!(mat.texture.is_none());
    }
}
