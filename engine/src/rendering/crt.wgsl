#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct CrtMaterial {
    scanline_intensity: f32,
    barrel_distortion: f32,
    color_bleeding: f32,
    enabled: u32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: CrtMaterial;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var screen_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var screen_sampler: sampler;

fn barrel_distort(uv: vec2<f32>, amount: f32) -> vec2<f32> {
    let centered = uv - vec2<f32>(0.5);
    let r2 = dot(centered, centered);
    let distorted = centered * (1.0 + amount * r2);
    return distorted + vec2<f32>(0.5);
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    var uv = mesh.uv;

    // Pass-through when CRT is disabled
    if (material.enabled == 0u) {
        return textureSample(screen_texture, screen_sampler, uv);
    }

    // Barrel distortion
    uv = barrel_distort(uv, material.barrel_distortion);

    // Clip outside the distorted area
    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    // Color bleeding: slight RGB channel offset
    let bleed = material.color_bleeding * 0.002;
    let r = textureSample(screen_texture, screen_sampler, uv + vec2<f32>(bleed, 0.0)).r;
    let g = textureSample(screen_texture, screen_sampler, uv).g;
    let b = textureSample(screen_texture, screen_sampler, uv - vec2<f32>(bleed, 0.0)).b;
    var color = vec4<f32>(r, g, b, 1.0);

    // Scanlines: darken every other row based on texture resolution
    let resolution = vec2<f32>(textureDimensions(screen_texture));
    let scanline = sin(uv.y * resolution.y * 3.14159) * 0.5 + 0.5;
    let scanline_factor = 1.0 - material.scanline_intensity * (1.0 - scanline);
    color = vec4<f32>(color.rgb * scanline_factor, 1.0);

    // Slight vignette at edges
    let vignette_uv = uv * (1.0 - uv);
    let vignette = clamp(vignette_uv.x * vignette_uv.y * 15.0, 0.0, 1.0);
    color = vec4<f32>(color.rgb * (0.8 + 0.2 * vignette), 1.0);

    return color;
}
