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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FadeDirection {
    In,
    Out,
}

/// Attach to a BGM entity to drive a volume fade over time.
#[derive(Component)]
pub struct BgmFade {
    pub direction: FadeDirection,
    pub elapsed: f32,
    pub duration: f32,
    pub from_volume: f32,
    pub to_volume: f32,
}

impl BgmFade {
    pub fn fade_in(duration: f32, target_volume: f32) -> Self {
        Self {
            direction: FadeDirection::In,
            elapsed: 0.0,
            duration,
            from_volume: 0.0,
            to_volume: target_volume,
        }
    }

    pub fn fade_out(duration: f32, current_volume: f32) -> Self {
        Self {
            direction: FadeDirection::Out,
            elapsed: 0.0,
            duration,
            from_volume: current_volume,
            to_volume: 0.0,
        }
    }

    pub fn progress(&self) -> f32 {
        (self.elapsed / self.duration).clamp(0.0, 1.0)
    }

    pub fn current_volume(&self) -> f32 {
        let t = self.progress();
        self.from_volume + (self.to_volume - self.from_volume) * t
    }

    pub fn is_complete(&self) -> bool {
        self.elapsed >= self.duration
    }
}

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

    #[test]
    fn test_bgm_fade_in_starts_silent() {
        let fade = BgmFade::fade_in(1.0, 0.8);
        assert_eq!(fade.current_volume(), 0.0);
        assert_eq!(fade.direction, FadeDirection::In);
    }

    #[test]
    fn test_bgm_fade_out_starts_loud() {
        let fade = BgmFade::fade_out(1.0, 0.8);
        assert!((fade.current_volume() - 0.8).abs() < f32::EPSILON);
        assert_eq!(fade.direction, FadeDirection::Out);
    }

    #[test]
    fn test_bgm_fade_progress() {
        let mut fade = BgmFade::fade_in(2.0, 1.0);
        assert_eq!(fade.progress(), 0.0);
        assert_eq!(fade.current_volume(), 0.0);

        fade.elapsed = 1.0;
        assert!((fade.progress() - 0.5).abs() < f32::EPSILON);
        assert!((fade.current_volume() - 0.5).abs() < f32::EPSILON);

        fade.elapsed = 2.0;
        assert!((fade.progress() - 1.0).abs() < f32::EPSILON);
        assert!((fade.current_volume() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_bgm_fade_complete() {
        let mut fade = BgmFade::fade_out(1.0, 0.5);
        assert!(!fade.is_complete());
        fade.elapsed = 0.5;
        assert!(!fade.is_complete());
        fade.elapsed = 1.0;
        assert!(fade.is_complete());
        fade.elapsed = 1.5;
        assert!(fade.is_complete());
    }

    #[test]
    fn test_bgm_fade_clamps_progress() {
        let mut fade = BgmFade::fade_in(1.0, 1.0);
        fade.elapsed = 5.0;
        assert!((fade.progress() - 1.0).abs() < f32::EPSILON);
        assert!((fade.current_volume() - 1.0).abs() < f32::EPSILON);
    }
}

impl Plugin for DJAudioPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AudioState::new())
            .register_type::<AudioState>()
            .register_type::<AudioCommand>()
            .add_message::<AudioCommand>()
            .add_systems(Update, (handle_audio_commands, tick_bgm_fades));

        use crate::contracts::{AppContractExt, ContractEntry, PluginContract};
        app.register_contract(PluginContract {
            name: "DJAudioPlugin".into(),
            description: "BGM and SFX playback with crossfade support".into(),
            resources: vec![ContractEntry::of::<AudioState>(
                "Current audio playback state",
            )],
            components: vec![
                ContractEntry::of::<BgmSource>("Marks entity as BGM audio source"),
                ContractEntry::of::<SfxSource>("Marks entity as SFX audio source"),
                ContractEntry::of::<BgmFade>("Drives volume fade over time"),
            ],
            events: vec![ContractEntry::of::<AudioCommand>("Audio control commands")],
            system_sets: vec![],
        });

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
            AudioCommand::PlayBgm { track, crossfade } => {
                let target_vol = audio_state.bgm_output_volume();
                if *crossfade > 0.0 {
                    for entity in bgm_query.iter() {
                        commands
                            .entity(entity)
                            .insert(BgmFade::fade_out(*crossfade, target_vol))
                            .remove::<BgmSource>();
                    }
                    let audio_handle: Handle<AudioSource> = asset_server.load(track.clone());
                    let mut settings = PlaybackSettings::LOOP;
                    settings.volume = bevy::audio::Volume::Linear(0.0);
                    commands.spawn((
                        AudioPlayer::<AudioSource>(audio_handle),
                        settings,
                        BgmSource,
                        BgmFade::fade_in(*crossfade, target_vol),
                    ));
                } else {
                    for entity in bgm_query.iter() {
                        commands.entity(entity).despawn();
                    }
                    let audio_handle: Handle<AudioSource> = asset_server.load(track.clone());
                    let mut settings = PlaybackSettings::LOOP;
                    settings.volume = bevy::audio::Volume::Linear(target_vol);
                    commands.spawn((
                        AudioPlayer::<AudioSource>(audio_handle),
                        settings,
                        BgmSource,
                    ));
                }
                audio_state.current_bgm = Some(track.clone());
                info!("Playing BGM: {}", track);
            }
            AudioCommand::StopBgm { fade_out } => {
                if *fade_out > 0.0 {
                    let target_vol = audio_state.bgm_output_volume();
                    for entity in bgm_query.iter() {
                        commands
                            .entity(entity)
                            .insert(BgmFade::fade_out(*fade_out, target_vol));
                    }
                } else {
                    for entity in bgm_query.iter() {
                        commands.entity(entity).despawn();
                    }
                }
                audio_state.current_bgm = None;
                info!("Stopped BGM");
            }
            AudioCommand::PlaySfx { sound } => {
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

/// Ticks active BGM fades, interpolating volume each frame.
fn tick_bgm_fades(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut BgmFade, &mut AudioSink)>,
) {
    let dt = time.delta_secs();
    for (entity, mut fade, mut sink) in query.iter_mut() {
        fade.elapsed += dt;
        sink.set_volume(bevy::audio::Volume::Linear(fade.current_volume()));

        if fade.is_complete() {
            match fade.direction {
                FadeDirection::Out => {
                    commands.entity(entity).despawn();
                }
                FadeDirection::In => {
                    commands.entity(entity).remove::<BgmFade>();
                }
            }
        }
    }
}
