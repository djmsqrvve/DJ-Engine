mod sequencer;
mod wav;

use bevy::prelude::*;
use midly::MidiMessage;
use std::collections::HashMap;

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
            .add_systems(Startup, (wav::setup_midi_assets, wav::load_overworld_midi))
            .add_systems(
                Update,
                (sequencer::handle_midi_commands, sequencer::midi_sequencer),
            );

        info!("MIDI Plugin initialized");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_playback_default() {
        let p = MidiPlayback::default();
        assert!(!p.is_playing);
        assert_eq!(p.current_tick, 0.0);
        assert_eq!(p.event_index, 0);
        assert!(!p.loop_playback);
    }

    #[test]
    fn test_midi_manager_default() {
        let m = MidiManager::default();
        assert!(m.active_voices.is_empty());
    }

    #[test]
    fn test_sequencer_event_fields() {
        let event = SequencerEvent {
            tick: 480,
            kind: SequencerEventKind::Tempo(500_000),
        };
        assert_eq!(event.tick, 480);
        if let SequencerEventKind::Tempo(bpm) = event.kind {
            assert_eq!(bpm, 500_000);
        } else {
            panic!("expected Tempo");
        }
    }
}
