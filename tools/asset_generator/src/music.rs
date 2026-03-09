use midly::{Format, Header, MetaMessage, MidiMessage, Smf, TrackEvent, TrackEventKind};

pub fn generate_overworld_theme() -> Vec<u8> {
    let mut smf = Smf::new(Header::new(
        Format::Parallel,
        midly::Timing::Metrical(480.into()),
    ));

    // Scale: D Harmonic Minor
    // D, E, F, G, A, Bb, C#, D
    // 62, 64, 65, 67, 69, 70, 73, 74
    let _scale_d_harmonic_minor = [62, 64, 65, 67, 69, 70, 73, 74];

    // Track 0: Bass (Square Wave - Program 80/81)
    let mut track0 = Vec::new();
    // Set Tempo (120 BPM = 500,000 microseconds per beat)
    track0.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(MetaMessage::Tempo(500_000.into())),
    });
    // Program Change (Bass, Ch 0) -> 80 (Square Lead) or 38 (Synth Bass 1)
    // Let's use 80 (Square) for retro vibe.
    track0.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Midi {
            channel: 0.into(),
            message: MidiMessage::ProgramChange { program: 80.into() },
        },
    });

    // Bass line: D2 (38), A2 (45), Bb2 (46), A2 (45) - Ostinato
    let bass_notes = [38, 45, 46, 45];
    for _ in 0..8 {
        // 8 measures
        for note in bass_notes.iter() {
            // Note On
            track0.push(TrackEvent {
                delta: 0.into(),
                kind: TrackEventKind::Midi {
                    channel: 0.into(),
                    message: MidiMessage::NoteOn {
                        key: (*note).into(),
                        vel: 100.into(),
                    },
                },
            });
            // Note Off (Quarter note = 480 ticks)
            track0.push(TrackEvent {
                delta: 480.into(),
                kind: TrackEventKind::Midi {
                    channel: 0.into(),
                    message: MidiMessage::NoteOff {
                        key: (*note).into(),
                        vel: 0.into(),
                    },
                },
            });
        }
    }
    track0.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });
    smf.tracks.push(track0);

    // Track 1: Chords/Pads (Sine/Pad - Program 88 "New Age Pad")
    let mut track1 = Vec::new();
    track1.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Midi {
            channel: 1.into(),
            message: MidiMessage::ProgramChange { program: 88.into() },
        },
    });

    // Chords: Dmin (D, F, A), Gm (G, Bb, D), A7 (A, C#, E, G)
    // Progression: i - iv - V7 - i
    // Each chord lasts 1 bar (4 * 480 = 1920 ticks).
    let chords = [
        vec![62, 65, 69], // Dmin
        vec![67, 70, 74], // Gm (inv?) actually G3, Bb3, D4. 55, 58, 62? Let's stay in middle register.
        // G3=55, Bb3=58, D4=62.
        vec![55, 58, 62],
        vec![57, 61, 64, 67], // A7 (A3, C#4, E4, G4)
        vec![62, 65, 69],     // Dmin
    ];

    for _ in 0..2 {
        // Repeat progression chords twice
        for chord in chords.iter() {
            // Arpeggiate slightly? No, block chords for pad.
            // Note On all
            for (i, note) in chord.iter().enumerate() {
                track1.push(TrackEvent {
                    delta: if i == 0 { 0.into() } else { 0.into() }, // Simultaneous
                    kind: TrackEventKind::Midi {
                        channel: 1.into(),
                        message: MidiMessage::NoteOn {
                            key: (*note).into(),
                            vel: 80.into(),
                        },
                    },
                });
            }
            // Note Off all after 1 bar
            for (i, note) in chord.iter().enumerate() {
                track1.push(TrackEvent {
                    delta: if i == 0 { 1920.into() } else { 0.into() },
                    kind: TrackEventKind::Midi {
                        channel: 1.into(),
                        message: MidiMessage::NoteOff {
                            key: (*note).into(),
                            vel: 0.into(),
                        },
                    },
                });
            }
        }
    }
    track1.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });
    smf.tracks.push(track1);

    // Track 2: Melody (Lead - Program 81 Sawtooth or 13 Xylophone for spooky)
    // Let's go Spooky: Celesta (8) or Music Box (10)?
    // User requested DKC2 + MapleStory + Spooky.
    // DKC2 uses lots of atmospheric synths. MapleStory uses catchy leads.
    // Let's try Program 11 (Vibraphone) or 13 (Xylophone) for the spooky/plucky feel.
    let mut track2 = Vec::new();
    track2.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Midi {
            channel: 2.into(),
            message: MidiMessage::ProgramChange { program: 13.into() },
        },
    });

    // Melody Logic: Arpeggios and gaps.
    // Ticks per 16th note = 120.
    // Dmin scale run.
    let melody_notes = [74, 73, 69, 67, 65, 64, 62, 61]; // descending run C# to C#? wait, 61 is C#.
                                                         // Just some random eerie notes.
    for _ in 0..8 {
        for note in melody_notes.iter() {
            track2.push(TrackEvent {
                delta: 240.into(), // 8th notes
                kind: TrackEventKind::Midi {
                    channel: 2.into(),
                    message: MidiMessage::NoteOn {
                        key: (*note).into(),
                        vel: 90.into(),
                    },
                },
            });
            track2.push(TrackEvent {
                delta: 240.into(),
                kind: TrackEventKind::Midi {
                    channel: 2.into(),
                    message: MidiMessage::NoteOff {
                        key: (*note).into(),
                        vel: 0.into(),
                    },
                },
            });
        }
    }
    track2.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });
    smf.tracks.push(track2);

    let mut buffer = Vec::new();
    smf.write(&mut buffer).unwrap();
    buffer
}
