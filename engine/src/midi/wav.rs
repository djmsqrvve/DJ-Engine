use super::{MidiAssets, MidiFileAsset, MidiLoadState, MidiPlayback, MidiSequence};
use bevy::asset::{io::Reader, AssetLoader, LoadContext};
use bevy::prelude::*;
use bevy::reflect::TypePath;
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

pub(super) fn parse_midi_bytes(
    bytes: &[u8],
) -> Result<(Vec<super::SequencerEvent>, u16, u32), String> {
    let smf = Smf::parse(bytes).map_err(|e| format!("Failed to parse MIDI: {e}"))?;

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
                        MidiMessage::NoteOn { key, vel } => Some(MidiMessage::NoteOn { key, vel }),
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

    Ok((events, ticks_per_beat, duration))
}

#[derive(Default, TypePath)]
pub struct MidiLoader;

impl AssetLoader for MidiLoader {
    type Asset = MidiFileAsset;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<MidiFileAsset, Self::Error> {
        let mut bytes = Vec::new();
        bevy::asset::AsyncReadExt::read_to_end(reader, &mut bytes).await?;
        let (events, ticks_per_beat, duration_ticks) = parse_midi_bytes(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(MidiFileAsset {
            events,
            ticks_per_beat,
            duration_ticks,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["mid"]
    }
}

pub(super) fn start_midi_load(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handle = asset_server.load::<MidiFileAsset>("music/overworld_theme.mid");
    commands.insert_resource(MidiLoadState(handle));
}

pub(super) fn apply_loaded_midi(
    mut commands: Commands,
    load_state: Option<Res<MidiLoadState>>,
    midi_assets: Res<Assets<MidiFileAsset>>,
) {
    let Some(load_state) = load_state else {
        return;
    };
    let Some(asset) = midi_assets.get(&load_state.0) else {
        return;
    };

    commands.insert_resource(MidiSequence {
        events: asset.events.clone(),
        ticks_per_beat: asset.ticks_per_beat,
        duration_ticks: asset.duration_ticks,
    });
    commands.insert_resource(MidiPlayback {
        is_playing: true,
        current_tick: 0.0,
        event_index: 0,
        microseconds_per_beat: 500_000,
        ticks_per_beat: asset.ticks_per_beat,
        loop_playback: true,
    });
    commands.remove_resource::<MidiLoadState>();
    info!("MIDI playback started (async)!");
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

    #[test]
    fn test_parse_midi_bytes_valid() {
        let midi_path = "games/dev/doomexe/assets/music/overworld_theme.mid";
        if let Ok(bytes) = std::fs::read(midi_path) {
            let result = parse_midi_bytes(&bytes);
            assert!(
                result.is_ok(),
                "parse_midi_bytes failed: {:?}",
                result.err()
            );
            let (events, ticks_per_beat, duration) = result.unwrap();
            assert!(!events.is_empty(), "expected MIDI events");
            assert!(ticks_per_beat > 0, "expected positive ticks_per_beat");
            assert!(duration > 0, "expected positive duration");
        }
        // Skip test if file not present (e.g. running from non-workspace root)
    }

    #[test]
    fn test_parse_midi_bytes_invalid() {
        let result = parse_midi_bytes(b"not a midi file");
        assert!(result.is_err());
    }
}
