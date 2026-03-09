use image::io::Reader as ImageReader;
use image::Rgba;
use std::path::Path;

mod music;

fn main() {
    // Assuming running from workspace root
    let assets_dir = Path::new("games/dev/doomexe/assets/sprites/hamster_parts");
    let music_dir = Path::new("games/dev/doomexe/assets/music");
    std::fs::create_dir_all(music_dir).expect("Failed to create music dir");

    // Generate Overworld Theme
    let midi_bytes = music::generate_overworld_theme();
    let midi_path = music_dir.join("overworld_theme.mid");
    std::fs::write(&midi_path, midi_bytes).expect("Failed to write MIDI file");
    println!("Generated MIDI: {:?}", midi_path);

    // Process existing files (which are actually JPEGs right now)
    // ... rest of processing
    // Body: 1024x1024 -> 64x64
    if assets_dir.exists() {
        process_asset(assets_dir.join("body/body.png"), 64, 64);

        // Heads: 1024x1024 -> 32x32
        process_asset(assets_dir.join("head/head.png"), 32, 32);
        process_asset(assets_dir.join("head/head_happy.png"), 32, 32);
        process_asset(assets_dir.join("head/head_angry.png"), 32, 32);

        // Limbs: 1024x1024 -> 16x16
        process_asset(assets_dir.join("body/paw_left.png"), 16, 16);
        process_asset(assets_dir.join("body/paw_right.png"), 16, 16);
        process_asset(assets_dir.join("body/foot_left.png"), 16, 16);
        process_asset(assets_dir.join("body/foot_right.png"), 16, 16);
    }
}

fn process_asset(path: std::path::PathBuf, w: u32, h: u32) {
    if !path.exists() {
        println!("Skipping missing file: {:?}", path);
        return;
    }

    println!("Processing {:?}...", path);

    // Read file bytes
    let bytes = std::fs::read(&path).expect("Failed to read file");

    // Load image guessing format from bytes
    let img = match ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .expect("Failed to guess format")
        .decode()
    {
        Ok(i) => i,
        Err(e) => {
            println!("Failed to decode image {:?}: {}", path, e);
            return;
        }
    };

    // Resize
    let resized = img.resize_exact(w, h, image::imageops::FilterType::Nearest);
    let mut rgba = resized.to_rgba8();

    // Chroma key (make black/dark background transparent)
    // AI pixel art usually has a solid background. Let's assume near-black.
    for pixel in rgba.pixels_mut() {
        let Rgba([r, g, b, _]) = *pixel;
        // Threshold for "black" background (adjust if needed)
        if r < 30 && g < 30 && b < 30 {
            *pixel = Rgba([0, 0, 0, 0]);
        } else {
            // Ensure full opacity for non-background
            *pixel = Rgba([r, g, b, 255]);
        }
    }

    // Save as proper PNG
    rgba.save(&path).expect("Failed to save fixed PNG");
    println!("Fixed and saved {:?}", path);
}
