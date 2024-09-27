
use crate::EqTunerModeEnum;

// mode Equalizer processing
mod audio_fft_binner;
mod visual_bins_to_animation;

// mode Tuner processing
mod audio_tuner;
mod visual_tuner_painter;

// visual elements and rendering
pub mod graphics;

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

    pub fn process(&mut self, audio_value: f32, mode: &EqTunerModeEnum) {
        let gain = 2.0; // linejack gives a fairly low amplitude. Note: tuner gains the signal some more because guitar also gives a low amplitude output.
        let audio_value_for_process = (audio_value*gain) as f32;

        match mode {
            EqTunerModeEnum::Equalizer => {
                self.frequalizer.frequalize(audio_value_for_process, self.sample_rate)
            },
            EqTunerModeEnum::Tuner => {
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
            eq_painter: visual_bins_to_animation::Painter::new(),
            tuner_painter: visual_tuner_painter::Painter{}
        }
    }

    pub fn process(&mut self, audio_processor: &AudioProcessor, mode: &EqTunerModeEnum) {
        match mode {
            EqTunerModeEnum::Equalizer => {
                self.color_vec = self.eq_painter.paint(&audio_processor.frequalizer.eq_bins);
            },
            EqTunerModeEnum::Tuner => {
                // note_info is optional because the pitch detector is strict
                if let Some(note_info) = &audio_processor.tuner.note_info {
                    self.color_vec = self.tuner_painter.paint(note_info);
                }
            }
        }
    }
}