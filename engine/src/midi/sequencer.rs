use super::{MidiAssets, MidiCommand, MidiManager, MidiPlayback, MidiSequence, SequencerEventKind};
use crate::audio::AudioState;
use bevy::audio::PlaybackMode;
use bevy::prelude::*;
use midly::MidiMessage;

pub(super) fn midi_sequencer(
    time: Res<Time>,
    sequence: Option<Res<MidiSequence>>,
    mut playback: ResMut<MidiPlayback>,
    mut commands: MessageWriter<MidiCommand>,
) {
    let Some(sequence) = sequence else { return };
    if !playback.is_playing {
        return;
    }

    let delta_secs = time.delta_secs_f64();
    let ticks_per_sec =
        (1_000_000.0 / playback.microseconds_per_beat as f64) * playback.ticks_per_beat as f64;
    let delta_ticks = delta_secs * ticks_per_sec;

    playback.current_tick += delta_ticks;

    while playback.event_index < sequence.events.len() {
        let event = &sequence.events[playback.event_index];
        if event.tick as f64 <= playback.current_tick {
            match event.kind {
                SequencerEventKind::Midi { message } => match message {
                    MidiMessage::NoteOn { key, vel } => {
                        commands.write(MidiCommand::NoteOn {
                            key: key.as_int(),
                            velocity: vel.as_int(),
                        });
                    }
                    MidiMessage::NoteOff { key, .. } => {
                        commands.write(MidiCommand::NoteOff { key: key.as_int() });
                    }
                    _ => {}
                },
                SequencerEventKind::Tempo(mpb) => {
                    playback.microseconds_per_beat = mpb;
                }
            }
            playback.event_index += 1;
        } else {
            break;
        }
    }

    if playback.event_index >= sequence.events.len() && playback.loop_playback {
        playback.current_tick = 0.0;
        playback.event_index = 0;
    }
}

pub(super) fn handle_midi_commands(
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
                let base_freq = 261.63;
                let speed = note_freq / base_freq;
                let volume = (*velocity as f32 / 127.0).clamp(0.0, 1.0)
                    * audio_state.master_volume
                    * audio_state.bgm_volume;

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
