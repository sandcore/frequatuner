use std::i32;

use crate::EqTunerMode;

// mode Equalizer
mod audio_fft_binner;
mod visual_bins_to_animation;

// mode Tuner
mod audio_tuner;
mod visual_tuner_painter;

// The audioprocessor fills either the buffer for equalizer or the buffer for tuner depending on mode
pub struct AudioProcessor {
    sample_rate: u32,
    frequalizer: audio_fft_binner::AudioFrequalizer,
    tuner: audio_tuner::GiTuner
}
impl AudioProcessor {
    pub fn new(buffer_length: usize, num_bins: u8, sample_rate: u32) -> Self {
        AudioProcessor {
            sample_rate,
            frequalizer: audio_fft_binner::AudioFrequalizer::new(num_bins),
            tuner: audio_tuner::GiTuner::new(buffer_length)
        }
    }

    pub fn process(&mut self, unprocessed_audio_value: i32, mode: &EqTunerMode) {
        // getting 4 byte i32 values
        let audio_value = unprocessed_audio_value as f64 / (i32::MAX) as f64; // normalized, between 0 and 1
        let gain = 2.0; // linejack signal has a fairly low amplitude. Note: tuner gains the signal some more because guitar also gives a low amplitude input.

        let audio_value_for_process = (audio_value*gain) as f32;

        match mode {
            EqTunerMode::Equalizer => {
                self.frequalizer.frequalize(audio_value_for_process, self.sample_rate)
            },
            EqTunerMode::Tuner => {
                self.tuner.tune(audio_value_for_process, self.sample_rate)
            }
        }
    }
}
pub struct VisualProcessor {
    pub color_vec: Vec<u8>,
    eq_painter: visual_bins_to_animation::Painter,
    tuner_painter: visual_tuner_painter::Painter
}
impl VisualProcessor {
    pub fn new(x:usize, y:usize) -> Self {
        VisualProcessor {
            color_vec: vec![],
            eq_painter: visual_bins_to_animation::Painter::new(x, y),
            tuner_painter: visual_tuner_painter::Painter::new(x,y)
        }
    }

    pub fn process(&mut self, audio_processor: &AudioProcessor, mode: &EqTunerMode) {
        match mode {
            EqTunerMode::Equalizer => {
                self.color_vec = self.eq_painter.paint(&audio_processor.frequalizer.eq_bins);
            },
            EqTunerMode::Tuner => {
                // note_info is optional because the pitch detector is sensitive
                if let Some(note_info) = &audio_processor.tuner.note_info {
                    self.color_vec = self.tuner_painter.paint(note_info);
                }
            }
        }
    }
}