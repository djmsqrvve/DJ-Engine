//! Offscreen rendering pipeline for pixel-perfect output.
//!
//! Renders the game world to a 320x240 texture, then displays it
//! upscaled to the window via a display camera on a separate render layer.
//! Optionally applies CRT post-processing effects.

use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::RenderLayers;
use bevy::camera::{ImageRenderTarget, RenderTarget};
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::sprite_render::MeshMaterial2d;

use super::camera::{MainCamera, GAME_HEIGHT, GAME_WIDTH};
use super::crt_material::{CrtMaterial, CrtParams};

#[derive(Component)]
pub struct DisplayCamera;

#[derive(Component)]
pub struct DisplayQuad;

#[derive(Resource)]
pub struct OffscreenTarget {
    pub image: Handle<Image>,
}

#[derive(Resource)]
pub struct CrtMaterialHandle(pub Handle<CrtMaterial>);

#[derive(Resource, Reflect, Clone)]
#[reflect(Resource)]
pub struct CrtConfig {
    pub enabled: bool,
    pub scanline_intensity: f32,
    pub barrel_distortion: f32,
    pub color_bleeding: f32,
}

impl Default for CrtConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            scanline_intensity: 0.3,
            barrel_distortion: 0.02,
            color_bleeding: 0.5,
        }
    }
}

pub fn setup_offscreen_pipeline(
    mut commands: Commands,
    images: Option<ResMut<Assets<Image>>>,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    mut crt_materials: Option<ResMut<Assets<CrtMaterial>>>,
) {
    let (Some(mut images), Some(ref mut meshes), Some(ref mut crt_materials)) =
        (images, meshes.as_mut(), crt_materials.as_mut())
    else {
        info!("Offscreen pipeline skipped (missing asset resources)");
        return;
    };

    let size = Extent3d {
        width: GAME_WIDTH as u32,
        height: GAME_HEIGHT as u32,
        depth_or_array_layers: 1,
    };

    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
        | TextureUsages::COPY_DST
        | TextureUsages::RENDER_ATTACHMENT;
    image.sampler = ImageSampler::nearest();

    let image_handle = images.add(image);

    commands.insert_resource(OffscreenTarget {
        image: image_handle.clone(),
    });

    // Game camera: renders the world to the 320x240 texture
    commands.spawn((
        Camera2d,
        Camera {
            order: -1,
            ..default()
        },
        RenderTarget::Image(ImageRenderTarget {
            handle: image_handle.clone(),
            scale_factor: 1.0,
        }),
        Projection::from(OrthographicProjection::default_2d()),
        MainCamera,
        Transform::from_xyz(0.0, 0.0, 1000.0),
    ));

    // CRT material with the render texture
    let crt_material = crt_materials.add(CrtMaterial {
        params: CrtParams::default(),
        texture: Some(image_handle),
    });
    commands.insert_resource(CrtMaterialHandle(crt_material.clone()));

    // Display quad: fullscreen rectangle with CRT material, on layer 1
    let quad_mesh = meshes.add(Rectangle::new(GAME_WIDTH, GAME_HEIGHT));
    commands.spawn((
        Mesh2d(quad_mesh),
        MeshMaterial2d(crt_material),
        Transform::default(),
        RenderLayers::layer(1),
        DisplayQuad,
    ));

    // Display camera: renders the quad to the window, layer 1
    commands.spawn((
        Camera2d,
        Camera {
            order: 0,
            ..default()
        },
        Projection::from(OrthographicProjection {
            scale: GAME_HEIGHT / 600.0,
            ..OrthographicProjection::default_2d()
        }),
        RenderLayers::layer(1),
        DisplayCamera,
    ));

    info!(
        "Offscreen pipeline: {}x{} render target -> window (CRT material ready)",
        GAME_WIDTH as u32, GAME_HEIGHT as u32
    );
}

pub fn sync_crt_config(
    config: Res<CrtConfig>,
    handle: Option<Res<CrtMaterialHandle>>,
    mut materials: Option<ResMut<Assets<CrtMaterial>>>,
) {
    let (Some(handle), Some(ref mut materials)) = (handle, materials.as_mut()) else {
        return;
    };
    if !config.is_changed() {
        return;
    }
    if let Some(mat) = materials.get_mut(&handle.0) {
        mat.params = CrtParams {
            scanline_intensity: config.scanline_intensity,
            barrel_distortion: config.barrel_distortion,
            color_bleeding: config.color_bleeding,
            enabled: if config.enabled { 1 } else { 0 },
        };
    }
}

pub fn resize_display_projection(
    windows: Query<&Window>,
    mut display_cam: Query<&mut Projection, With<DisplayCamera>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok(mut projection) = display_cam.single_mut() else {
        return;
    };

    if let Projection::Orthographic(ortho) = &mut *projection {
        let scale_x = GAME_WIDTH / window.resolution.width();
        let scale_y = GAME_HEIGHT / window.resolution.height();
        ortho.scale = scale_x.max(scale_y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crt_config_defaults() {
        let config = CrtConfig::default();
        assert!(!config.enabled);
        assert!((config.scanline_intensity - 0.3).abs() < f32::EPSILON);
        assert!((config.barrel_distortion - 0.02).abs() < f32::EPSILON);
        assert!((config.color_bleeding - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_crt_params_from_config() {
        let config = CrtConfig {
            enabled: true,
            scanline_intensity: 0.5,
            barrel_distortion: 0.1,
            color_bleeding: 0.8,
        };
        let params = CrtParams {
            scanline_intensity: config.scanline_intensity,
            barrel_distortion: config.barrel_distortion,
            color_bleeding: config.color_bleeding,
            enabled: if config.enabled { 1 } else { 0 },
        };
        assert_eq!(params.enabled, 1);
        assert!((params.scanline_intensity - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_offscreen_target_dimensions() {
        assert_eq!(GAME_WIDTH as u32, 320);
        assert_eq!(GAME_HEIGHT as u32, 240);
    }
}
