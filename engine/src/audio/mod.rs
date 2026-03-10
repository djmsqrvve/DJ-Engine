//! Audio system for DJ Engine.
//!
//! Provides BGM and SFX playback with crossfade support for scene transitions.

use bevy::prelude::*;

/// Audio state resource tracking current playback.
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct AudioState {
    /// Currently playing BGM track name (if any)
    pub current_bgm: Option<String>,
    /// Master volume (0.0 - 1.0)
    pub master_volume: f32,
    /// BGM volume (0.0 - 1.0)
    pub bgm_volume: f32,
    /// SFX volume (0.0 - 1.0)
    pub sfx_volume: f32,
}

impl AudioState {
    pub fn new() -> Self {
        Self {
            current_bgm: None,
            master_volume: 0.0,
            bgm_volume: 0.8,
            sfx_volume: 1.0,
        }
    }

    fn bgm_output_volume(&self) -> f32 {
        self.master_volume * self.bgm_volume
    }

    fn sfx_output_volume(&self) -> f32 {
        self.master_volume * self.sfx_volume
    }
}

/// Messages for audio control.
#[derive(Message, Debug, Clone, PartialEq, Reflect)]
pub enum AudioCommand {
    /// Play background music (with optional crossfade duration in seconds)
    PlayBgm { track: String, crossfade: f32 },
    /// Stop current BGM (with optional fade out duration)
    StopBgm { fade_out: f32 },
    /// Play a one-shot sound effect
    PlaySfx { sound: String },
    /// Set master volume
    SetMasterVolume(f32),
    /// Set BGM volume
    SetBgmVolume(f32),
    /// Set SFX volume
    SetSfxVolume(f32),
}

/// Component marking an entity as the BGM audio source.
#[derive(Component)]
pub struct BgmSource;

/// Component marking an entity as an SFX audio source.
#[derive(Component)]
pub struct SfxSource;

/// Plugin providing audio functionality.
pub struct DJAudioPlugin;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_state_new_defaults() {
        let state = AudioState::new();
        assert_eq!(state.master_volume, 0.0);
        assert_eq!(state.bgm_volume, 0.8);
        assert_eq!(state.sfx_volume, 1.0);
        assert!(state.current_bgm.is_none());
    }

    #[test]
    fn test_bgm_output_volume() {
        let mut state = AudioState::new();
        state.master_volume = 0.5;
        state.bgm_volume = 0.8;
        assert!((state.bgm_output_volume() - 0.4).abs() < f32::EPSILON);

        state.master_volume = 0.0;
        assert_eq!(state.bgm_output_volume(), 0.0);

        state.master_volume = 1.0;
        state.bgm_volume = 1.0;
        assert_eq!(state.bgm_output_volume(), 1.0);
    }

    #[test]
    fn test_sfx_output_volume() {
        let mut state = AudioState::new();
        state.master_volume = 0.5;
        state.sfx_volume = 1.0;
        assert!((state.sfx_output_volume() - 0.5).abs() < f32::EPSILON);

        state.master_volume = 1.0;
        state.sfx_volume = 0.5;
        assert!((state.sfx_output_volume() - 0.5).abs() < f32::EPSILON);

        state.master_volume = 0.0;
        assert_eq!(state.sfx_output_volume(), 0.0);
    }
}

impl Plugin for DJAudioPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AudioState::new())
            .register_type::<AudioState>()
            .register_type::<AudioCommand>()
            .add_message::<AudioCommand>()
            .add_systems(Update, handle_audio_commands);

        info!("DJ Audio Plugin initialized");
    }
}

/// System that processes audio commands.
fn handle_audio_commands(
    mut commands: Commands,
    mut audio_commands: MessageReader<AudioCommand>,
    mut audio_state: ResMut<AudioState>,
    asset_server: Res<AssetServer>,
    bgm_query: Query<Entity, With<BgmSource>>,
) {
    for cmd in audio_commands.read() {
        match cmd {
            AudioCommand::PlayBgm {
                track,
                crossfade: _,
            } => {
                // Stop existing BGM first
                for entity in bgm_query.iter() {
                    commands.entity(entity).despawn();
                }

                // Load and play new BGM
                let audio_handle: Handle<AudioSource> = asset_server.load(track.clone());
                let mut settings = PlaybackSettings::LOOP;
                settings.volume = bevy::audio::Volume::Linear(audio_state.bgm_output_volume());
                commands.spawn((
                    AudioPlayer::<AudioSource>(audio_handle),
                    settings,
                    BgmSource,
                ));
                audio_state.current_bgm = Some(track.clone());
                info!("Playing BGM: {}", track);
            }
            AudioCommand::StopBgm { fade_out: _ } => {
                // Stop current BGM
                for entity in bgm_query.iter() {
                    commands.entity(entity).despawn();
                }
                audio_state.current_bgm = None;
                info!("Stopped BGM");
            }
            AudioCommand::PlaySfx { sound } => {
                // Play one-shot SFX
                let audio_handle: Handle<AudioSource> = asset_server.load(sound.clone());
                let mut settings = PlaybackSettings::DESPAWN;
                settings.volume = bevy::audio::Volume::Linear(audio_state.sfx_output_volume());
                commands.spawn((
                    AudioPlayer::<AudioSource>(audio_handle),
                    settings,
                    SfxSource,
                ));
                debug!("Playing SFX: {}", sound);
            }
            AudioCommand::SetMasterVolume(vol) => {
                audio_state.master_volume = vol.clamp(0.0, 1.0);
            }
            AudioCommand::SetBgmVolume(vol) => {
                audio_state.bgm_volume = vol.clamp(0.0, 1.0);
            }
            AudioCommand::SetSfxVolume(vol) => {
                audio_state.sfx_volume = vol.clamp(0.0, 1.0);
            }
        }
    }
}
