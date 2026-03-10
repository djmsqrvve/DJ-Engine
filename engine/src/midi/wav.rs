use super::{MidiAssets, MidiPlayback, MidiSequence};
use bevy::prelude::*;
use midly::{MetaMessage, MidiMessage, Smf, TrackEventKind};

pub(super) fn setup_midi_assets(mut commands: Commands, mut assets: ResMut<Assets<AudioSource>>) {
    let sample_rate = 44100;
    let duration_secs = 2;
    let num_samples = sample_rate * duration_secs;
    let frequency = 261.63;

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

pub(super) fn load_overworld_midi(mut commands: Commands) {
    let path = "games/dev/doomexe/assets/music/overworld_theme.mid";
    if let Ok(bytes) = std::fs::read(path) {
        if let Ok(smf) = Smf::parse(&bytes) {
            info!("Loaded MIDI: tracks={}", smf.tracks.len());

            let ticks_per_beat = match smf.header.timing {
                midly::Timing::Metrical(t) => t.as_int(),
                _ => 480,
            };

            let mut events = Vec::new();

            for track in smf.tracks {
                let mut current_tick = 0;
                for event in track {
                    current_tick += event.delta.as_int();
                    match event.kind {
                        TrackEventKind::Midi { message, .. } => {
                            let owned_msg = match message {
                                MidiMessage::NoteOn { key, vel } => {
                                    Some(MidiMessage::NoteOn { key, vel })
                                }
                                MidiMessage::NoteOff { key, vel } => {
                                    Some(MidiMessage::NoteOff { key, vel })
                                }
                                _ => None,
                            };

                            if let Some(msg) = owned_msg {
                                events.push(super::SequencerEvent {
                                    tick: current_tick,
                                    kind: super::SequencerEventKind::Midi { message: msg },
                                });
                            }
                        }
                        TrackEventKind::Meta(MetaMessage::Tempo(t)) => {
                            events.push(super::SequencerEvent {
                                tick: current_tick,
                                kind: super::SequencerEventKind::Tempo(t.as_int()),
                            });
                        }
                        _ => {}
                    }
                }
            }

            events.sort_by_key(|e| e.tick);
            let duration = events.last().map(|e| e.tick).unwrap_or(0);

            commands.insert_resource(MidiSequence {
                events,
                ticks_per_beat,
                duration_ticks: duration,
            });

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

pub(super) fn generate_wav(num_samples: u32, sample_rate: u32, samples: &[f32]) -> AudioSource {
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

pub(super) fn generate_wav_square(num_samples: u32, sample_rate: u32, freq: f32) -> AudioSource {
    let mut samples = Vec::with_capacity(num_samples as usize);
    for i in 0..num_samples {
        let t = (i as f32) / (sample_rate as f32);
        let phase = t * freq;
        let sample = if phase.fract() < 0.5 { 0.5 } else { -0.5 };
        samples.push(sample);
    }
    generate_wav(num_samples, sample_rate, &samples)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_wav_valid_header() {
        let samples: Vec<f32> = vec![0.0; 44100];
        let source = generate_wav(44100, 44100, &samples);
        let bytes = &source.bytes;
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
        assert_eq!(&bytes[12..16], b"fmt ");
        assert_eq!(&bytes[36..40], b"data");
    }

    #[test]
    fn test_generate_wav_square_valid_header() {
        let source = generate_wav_square(44100, 44100, 440.0);
        let bytes = &source.bytes;
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
    }
}
