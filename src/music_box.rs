use std::collections::HashMap;

struct NoteMeta {
    clock: f32,
    release_t: Option<f32>,
}

pub struct MusicBoxConfig {
    fade_in: f32,
    fade_out: f32,
}

pub struct MusicBox {
    notes: HashMap<u8, NoteMeta>,
    config: MusicBoxConfig,
    volume: f32,
}

impl MusicBox {
    pub fn new() -> MusicBox {
        MusicBox {
            notes: HashMap::new(),
            config: MusicBoxConfig {
                fade_in: 0.05,
                fade_out: 0.5,
            },
            volume: 0.5,
        }
    }

    pub fn increase_clock(&mut self, t: f32) {
        for (_note, meta) in self.notes.iter_mut() {
            (*meta).clock += t;
        }

        let fade_out = self.config.fade_out;
        self.notes.retain(|_note, meta| {
            meta.release_t
                .map(|release_t| meta.clock - release_t <= fade_out)
                .unwrap_or(true)
        })
    }

    pub fn press(&mut self, note: u8) {
        self.notes.insert(
            note,
            NoteMeta {
                clock: 0f32,
                release_t: None,
            },
        );
    }

    pub fn release(&mut self, note: u8) {
        if let Some(meta) = self.notes.remove(&note) {
            self.notes.insert(note, NoteMeta { release_t: Some(meta.clock), ..meta });
        }
    }

    pub fn set_volume(&mut self, level: u8) {
        self.volume = level as f32 / 127f32
    }

    pub fn get_sample(&self) -> f32 {
        let sample: f32 = self
            .notes
            .iter()
            // note number to frequency
            .map(|(note, meta)| {
                let freq = note_midi(440f32, *note);
                (freq, meta)
            })
            // frequency to sample
            .map(|(freq, meta)| {
                let sample = (meta.clock * freq * 2f32 * std::f32::consts::PI).sin();
                let sample = if sample < 0f32 { -0.2 } else if sample > 0f32 { 0.2 } else {0f32};
                (sample, meta)
            })
            // fade-in-out
            .map(|(sample, meta)| {
                let sample = sample * (meta.clock / self.config.fade_in).min(1f32);

                if let Some(release_t) = meta.release_t {
                    let release_dt = meta.clock - release_t;
                    sample * (1f32 - (release_dt / self.config.fade_out).min(1f32))
                } else {
                    sample
                }
            })
            .sum();
        sample * self.volume / 2f32
    }
}

fn note(a4: f32, semitone: u8, octave: i8) -> f32 {
    let semitones_from_a4 = octave as i32 * 12 + semitone as i32 - 9 - 48;
    a4 * (semitones_from_a4 as f32 * 2.0f32.ln() / 12.0).exp()
}

fn note_midi(a4: f32, midi_note: u8) -> f32 {
    let semitone = midi_note % 12;
    let octave = (midi_note as i16 / 12) as i8 - 1;
    note(a4, semitone, octave)
}
