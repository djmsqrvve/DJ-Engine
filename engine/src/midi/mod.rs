use bevy::audio::PlaybackMode;
use bevy::prelude::*;
use midly::{MetaMessage, MidiMessage, Smf, TrackEventKind};
use std::collections::HashMap;

use crate::audio::AudioState;

/// Events for controlling MIDI playback
#[derive(Message, Debug, Clone)]
pub enum MidiCommand {
    NoteOn { key: u8, velocity: u8 },
    NoteOff { key: u8 },
}

/// Resource to hold generated waveform assets
#[derive(Resource)]
pub struct MidiAssets {
    pub sine_wave: Handle<AudioSource>,
    pub square_wave: Handle<AudioSource>,
}

#[derive(Resource, Default)]
pub struct MidiManager {
    /// active notes: Key -> Entity (AudioSource)
    pub active_voices: HashMap<u8, Entity>,
}

/// A parsed, playable MIDI sequence (flattened for simplicity)
#[derive(Resource, Clone)]
pub struct MidiSequence {
    pub events: Vec<SequencerEvent>,
    pub ticks_per_beat: u16,
    pub duration_ticks: u32,
}

#[derive(Clone, Debug)]
pub struct SequencerEvent {
    pub tick: u32,
    pub kind: SequencerEventKind,
}

#[derive(Clone, Debug)]
pub enum SequencerEventKind {
    Midi { message: MidiMessage },
    Tempo(u32), // Microseconds per beat
}

/// Resource for playback state
#[derive(Resource, Default)]
pub struct MidiPlayback {
    pub is_playing: bool,
    pub current_tick: f64,
    pub event_index: usize,
    pub microseconds_per_beat: u32,
    pub ticks_per_beat: u16,
    pub loop_playback: bool,
}

pub struct MidiPlugin;

impl Plugin for MidiPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<MidiCommand>()
            .init_resource::<MidiManager>()
            .init_resource::<MidiPlayback>()
            .add_systems(Startup, (setup_midi_assets, load_overworld_midi))
            .add_systems(Update, (handle_midi_commands, midi_sequencer));

        info!("MIDI Plugin initialized");
    }
}

// ... (Audio setup code same as before, preserving it) ...
fn setup_midi_assets(mut commands: Commands, mut assets: ResMut<Assets<AudioSource>>) {
    // Generate 1 second of C4 (Middle C, ~261.63 Hz) at 44100Hz
    let sample_rate = 44100;
    let duration_secs = 2; // Loop length
    let num_samples = sample_rate * duration_secs;
    let frequency = 261.63;

    // Sine Wave
    let mut sine_buffer = Vec::with_capacity(num_samples);
    for i in 0..num_samples {
        let t = (i as f32) / (sample_rate as f32);
        let sample = (t * frequency * 2.0 * std::f32::consts::PI).sin();
        sine_buffer.push(sample);
    }

    let sine_source = generate_wav(num_samples as u32, sample_rate as u32, &sine_buffer);
    let sine_handle = assets.add(sine_source);

    let square_source = generate_wav_square(num_samples as u32, sample_rate as u32, frequency);
    let square_handle = assets.add(square_source);

    commands.insert_resource(MidiAssets {
        sine_wave: sine_handle,
        square_wave: square_handle,
    });
}

// Hardcoded load for MVP: Load the file we just generated
// In a real engine, this would be an AssetLoader.
fn load_overworld_midi(mut commands: Commands) {
    // Read file from assets folder using std::fs for now (blocking!) because we don't have a binary loader yet.
    // Assuming workspace structure.
    let path = "games/dev/doomexe/assets/music/overworld_theme.mid";
    if let Ok(bytes) = std::fs::read(path) {
        if let Ok(smf) = Smf::parse(&bytes) {
            info!("Loaded MIDI: tracks={}", smf.tracks.len());

            let ticks_per_beat = match smf.header.timing {
                midly::Timing::Metrical(t) => t.as_int(),
                _ => 480,
            };

            // Flatten tracks
            let mut events = Vec::new();

            for track in smf.tracks {
                let mut current_tick = 0;
                for event in track {
                    current_tick += event.delta.as_int();
                    match event.kind {
                        TrackEventKind::Midi { message, .. } => {
                            // Map message to owned
                            // Midly messages have lifetimes, need to convert to our own 'static types if needed.
                            // Or handle carefully. Midly 0.5 `MidiMessage` is Copy/Clone?
                            // `MidiMessage` has lifetime? Yes `MidiMessage<'a>`.
                            // We need to convert to a simpler owned enum or store raw bytes.
                            // For MVP, handling NoteOn/Off is enough.

                            let owned_msg = match message {
                                MidiMessage::NoteOn { key, vel } => {
                                    Some(MidiMessage::NoteOn { key, vel })
                                }
                                MidiMessage::NoteOff { key, vel } => {
                                    Some(MidiMessage::NoteOff { key, vel })
                                }
                                _ => None, // Ignore others for now
                            };

                            if let Some(msg) = owned_msg {
                                events.push(SequencerEvent {
                                    tick: current_tick,
                                    kind: SequencerEventKind::Midi { message: msg },
                                });
                            }
                        }
                        TrackEventKind::Meta(MetaMessage::Tempo(t)) => {
                            events.push(SequencerEvent {
                                tick: current_tick,
                                kind: SequencerEventKind::Tempo(t.as_int()),
                            });
                        }
                        _ => {}
                    }
                }
            }

            // Sort by tick
            events.sort_by_key(|e| e.tick);

            let duration = events.last().map(|e| e.tick).unwrap_or(0);

            commands.insert_resource(MidiSequence {
                events,
                ticks_per_beat,
                duration_ticks: duration,
            });

            // Start Playback
            commands.insert_resource(MidiPlayback {
                is_playing: true,
                current_tick: 0.0,
                event_index: 0,
                microseconds_per_beat: 500_000,
                ticks_per_beat,
                loop_playback: true,
            });
            info!("MIDI playback started!");
        } else {
            error!("Failed to parse MIDI");
        }
    } else {
        warn!("MIDI file not found at {}", path);
    }
}

fn midi_sequencer(
    time: Res<Time>,
    sequence: Option<Res<MidiSequence>>,
    mut playback: ResMut<MidiPlayback>,
    mut commands: MessageWriter<MidiCommand>,
) {
    let Some(sequence) = sequence else { return };
    if !playback.is_playing {
        return;
    }

    // Calculate ticks to advance
    // Seconds * (microsec/sec) / (microsec/beat) * (ticks/beat)
    let delta_secs = time.delta_secs_f64();
    let ticks_per_sec =
        (1_000_000.0 / playback.microseconds_per_beat as f64) * playback.ticks_per_beat as f64;
    let delta_ticks = delta_secs * ticks_per_sec;

    playback.current_tick += delta_ticks;

    // Process events
    while playback.event_index < sequence.events.len() {
        let event = &sequence.events[playback.event_index];
        if event.tick as f64 <= playback.current_tick {
            // Fire event
            match event.kind {
                SequencerEventKind::Midi { message } => {
                    match message {
                        MidiMessage::NoteOn { key, vel } => {
                            // Midly key/vel are wrappers.
                            commands.write(MidiCommand::NoteOn {
                                key: key.as_int(),
                                velocity: vel.as_int(),
                            });
                        }
                        MidiMessage::NoteOff { key, .. } => {
                            commands.write(MidiCommand::NoteOff { key: key.as_int() });
                        }
                        _ => {}
                    }
                }
                SequencerEventKind::Tempo(mpb) => {
                    playback.microseconds_per_beat = mpb;
                }
            }
            playback.event_index += 1;
        } else {
            break;
        }
    }

    // Looping ??
    if playback.event_index >= sequence.events.len() && playback.loop_playback {
        // Reset
        playback.current_tick = 0.0;
        playback.event_index = 0;
        // Note: active notes might get stuck if NoteOff was at the very end.
        // Real sequencer would silence all notes.
    }
}

// ... (Generate WAV / handle_midi_commands same as before) ...
fn generate_wav(num_samples: u32, sample_rate: u32, samples: &[f32]) -> AudioSource {
    let mut bytes = Vec::new();
    let data_len = num_samples * 2;
    let total_len = 36 + data_len;

    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&total_len.to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&(16u32).to_le_bytes());
    bytes.extend_from_slice(&(1u16).to_le_bytes());
    bytes.extend_from_slice(&(1u16).to_le_bytes());
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    bytes.extend_from_slice(&(2u16).to_le_bytes());
    bytes.extend_from_slice(&(16u16).to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_len.to_le_bytes());
    for sample in samples {
        let pcm = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        bytes.extend_from_slice(&pcm.to_le_bytes());
    }
    AudioSource {
        bytes: bytes.into(),
    }
}

fn generate_wav_square(num_samples: u32, sample_rate: u32, freq: f32) -> AudioSource {
    let mut samples = Vec::with_capacity(num_samples as usize);
    for i in 0..num_samples {
        let t = (i as f32) / (sample_rate as f32);
        let phase = t * freq;
        let sample = if phase.fract() < 0.5 { 0.5 } else { -0.5 };
        samples.push(sample);
    }
    generate_wav(num_samples, sample_rate, &samples)
}

fn handle_midi_commands(
    mut commands: Commands,
    mut events: MessageReader<MidiCommand>,
    mut manager: ResMut<MidiManager>,
    audio_state: Res<AudioState>,
    midi_assets: Option<Res<MidiAssets>>,
) {
    let Some(assets) = midi_assets else { return };

    for event in events.read() {
        match event {
            MidiCommand::NoteOn { key, velocity } => {
                let note_freq = 440.0 * 2.0_f32.powf((*key as f32 - 69.0) / 12.0);
                let base_freq = 261.63; // C4
                let speed = note_freq / base_freq;
                let volume = (*velocity as f32 / 127.0).clamp(0.0, 1.0)
                    * audio_state.master_volume
                    * audio_state.bgm_volume;

                // Use square wave for bass notes (< 55 G3 ?)
                let source = if *key < 55 {
                    assets.square_wave.clone()
                } else {
                    assets.sine_wave.clone()
                };

                let entity = commands
                    .spawn((
                        AudioPlayer(source),
                        PlaybackSettings {
                            mode: PlaybackMode::Loop,
                            speed,
                            volume: bevy::audio::Volume::Linear(volume),
                            ..default()
                        },
                    ))
                    .id();

                if let Some(old_entity) = manager.active_voices.insert(*key, entity) {
                    commands.entity(old_entity).despawn();
                }
            }
            MidiCommand::NoteOff { key } => {
                if let Some(entity) = manager.active_voices.remove(key) {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}
