//! Character assembly - spawning sprite hierarchies.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use super::components::*;

/// Helper to create a 1x1 colored texture.
fn create_color_texture(images: &mut Assets<Image>, color: Color) -> Handle<Image> {
    let size = Extent3d {
        width: 1,
        height: 1,
        depth_or_array_layers: 1,
    };
    let pixel = color.to_srgba().to_u8_array(); // 4 bytes [R, G, B, A]
    let image = Image::new_fill(
        size,
        TextureDimension::D2,
        &pixel,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    images.add(image)
}

/// Spawns a complete hamster character with all parts and animations.
pub fn spawn_character(commands: &mut Commands, images: &mut Assets<Image>) -> Entity {
    // Palette
    let color_body = Color::srgb(0.5, 0.3, 0.1); // Brown
    let color_head = Color::srgb(0.7, 0.5, 0.3); // Light Brown
    let color_paw = Color::srgb(0.8, 0.6, 0.5); // Pinkish
    let color_eye = Color::WHITE;
    let color_mouth_neutral = Color::BLACK;
    let color_mouth_happy = Color::srgb(0.8, 0.2, 0.2);
    let color_mouth_angry = Color::srgb(0.5, 0.0, 0.0);

    // Textures
    let tex_body = create_color_texture(images, color_body);
    let tex_head = create_color_texture(images, color_head);
    let tex_paw = create_color_texture(images, color_paw);
    let tex_eye = create_color_texture(images, color_eye);
    let tex_mouth_neutral = create_color_texture(images, color_mouth_neutral);
    let tex_mouth_happy = create_color_texture(images, color_mouth_happy);
    let tex_mouth_angry = create_color_texture(images, color_mouth_angry);

    let root = commands
        .spawn((
            CharacterRoot::new(),
            Transform::from_xyz(0.0, 0.0, 100.0),
            Visibility::default(),
        ))
        .id();

    // Body (Rectangle)
    let body = commands
        .spawn((
            Sprite {
                image: tex_body.clone(),
                custom_size: Some(Vec2::new(60.0, 80.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 100.0),
            SpritePart {
                kind: PartKind::Body,
                z_layer: 100,
            },
            BreathingAnimation::hamster_default(),
            CorruptionEffect::default(),
        ))
        .id();
    commands.entity(root).add_child(body);

    // Head (Rectangle)
    let head = commands
        .spawn((
            Sprite {
                image: tex_head.clone(),
                custom_size: Some(Vec2::new(50.0, 40.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 35.0, 101.0),
            SpritePart {
                kind: PartKind::Head,
                z_layer: 101,
            },
            IdleMotion {
                base_offset: Vec2::new(0.0, 35.0),
                amplitude: Vec2::new(2.0, 1.0),
                frequency: 0.12,
            },
            ExpressionSprite {
                neutral: tex_head.clone(),
                happy: tex_head.clone(), // Could vary color/size if desired
                angry: tex_head.clone(),
            },
            CorruptionEffect::default(),
        ))
        .id();
    commands.entity(root).add_child(head);

    // Left Eye
    let left_eye = commands
        .spawn((
            Sprite {
                image: tex_eye.clone(),
                custom_size: Some(Vec2::new(8.0, 8.0)),
                ..default()
            },
            Transform::from_xyz(-10.0, 5.0, 1.0),
            SpritePart {
                kind: PartKind::LeftEye,
                z_layer: 102,
            },
            BlinkingAnimation::default(),
            CorruptionEffect::default(),
        ))
        .id();
    commands.entity(head).add_child(left_eye);

    // Right Eye
    let right_eye = commands
        .spawn((
            Sprite {
                image: tex_eye.clone(),
                custom_size: Some(Vec2::new(8.0, 8.0)),
                ..default()
            },
            Transform::from_xyz(10.0, 5.0, 1.0),
            SpritePart {
                kind: PartKind::RightEye,
                z_layer: 102,
            },
            BlinkingAnimation::default(),
            CorruptionEffect::default(),
        ))
        .id();
    commands.entity(head).add_child(right_eye);

    // Mouth
    let mouth = commands
        .spawn((
            Sprite {
                image: tex_mouth_neutral.clone(),
                custom_size: Some(Vec2::new(12.0, 4.0)),
                ..default()
            },
            Transform::from_xyz(0.0, -10.0, 1.0),
            SpritePart {
                kind: PartKind::Mouth,
                z_layer: 102,
            },
            ExpressionSprite {
                neutral: tex_mouth_neutral.clone(),
                happy: tex_mouth_happy.clone(),
                angry: tex_mouth_angry.clone(),
            },
            CorruptionEffect::default(),
        ))
        .id();
    commands.entity(head).add_child(mouth);

    // Left Paw
    let left_paw = commands
        .spawn((
            Sprite {
                image: tex_paw.clone(),
                custom_size: Some(Vec2::new(15.0, 15.0)),
                ..default()
            },
            Transform::from_xyz(-25.0, -10.0, 102.0),
            SpritePart {
                kind: PartKind::LeftPaw,
                z_layer: 102,
            },
            CorruptionEffect::default(),
        ))
        .id();
    commands.entity(root).add_child(left_paw);

    // Right Paw
    let right_paw = commands
        .spawn((
            Sprite {
                image: tex_paw.clone(),
                custom_size: Some(Vec2::new(15.0, 15.0)),
                ..default()
            },
            Transform::from_xyz(25.0, -10.0, 102.0),
            SpritePart {
                kind: PartKind::RightPaw,
                z_layer: 102,
            },
            CorruptionEffect::default(),
        ))
        .id();
    commands.entity(root).add_child(right_paw);

    // Left Foot
    let left_foot = commands
        .spawn((
            Sprite {
                image: tex_paw.clone(),
                custom_size: Some(Vec2::new(18.0, 12.0)),
                ..default()
            },
            Transform::from_xyz(-15.0, -40.0, 99.0),
            SpritePart {
                kind: PartKind::LeftFoot,
                z_layer: 99,
            },
            CorruptionEffect::default(),
        ))
        .id();
    commands.entity(root).add_child(left_foot);

    // Right Foot
    let right_foot = commands
        .spawn((
            Sprite {
                image: tex_paw.clone(),
                custom_size: Some(Vec2::new(18.0, 12.0)),
                ..default()
            },
            Transform::from_xyz(15.0, -40.0, 99.0),
            SpritePart {
                kind: PartKind::RightFoot,
                z_layer: 99,
            },
            CorruptionEffect::default(),
        ))
        .id();
    commands.entity(root).add_child(right_foot);

    info!("🐹 Procedural Hamster spawned (Shapes only!)");

    root
}
