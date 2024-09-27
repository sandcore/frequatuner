
use pitch_detector::note::NoteDetectionResult;

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
    pub fn new(buffer_length: usize, num_bins: usize, sample_rate: &u32) -> Self {
        AudioProcessor {
            sample_rate: *sample_rate,
            frequalizer: audio_fft_binner::AudioFrequalizer::new(num_bins),
            tuner: audio_tuner::GiTuner::new(buffer_length)
        }
    }

    pub fn process(&mut self, audio_values: Vec<f32>, mode: &EqTunerModeEnum) {
        match mode {
            EqTunerModeEnum::Equalizer => {
                self.frequalizer.frequalize(audio_values, self.sample_rate)
            },
            EqTunerModeEnum::Tuner => {
                self.tuner.tune(audio_values, self.sample_rate)
            }
        }
    }
    
    pub fn output(&mut self, mode: &EqTunerModeEnum) -> AudioProcessorOutputEnum {
        match mode {
            EqTunerModeEnum::Equalizer => {
                AudioProcessorOutputEnum::EqBins(&self.frequalizer.eq_bins)
            },
            EqTunerModeEnum::Tuner => {
                // note_info is optional because the pitch detector is strict
                AudioProcessorOutputEnum::NoteInfo(&self.tuner.note_info)
            }
        }
    }
}

pub enum AudioProcessorOutputEnum<'a> {
    EqBins(&'a Vec<f32>),
    NoteInfo(&'a Option<NoteDetectionResult>)
}

pub struct VisualProcessor {
    eq_painter: visual_bins_to_animation::Painter,
    tuner_painter: visual_tuner_painter::Painter
}
impl VisualProcessor {
    pub fn new() -> Self {
        VisualProcessor {
            eq_painter: visual_bins_to_animation::Painter::new(),
            tuner_painter: visual_tuner_painter::Painter{}
        }
    }

    pub fn process(&mut self, input: AudioProcessorOutputEnum) -> Option<Vec<u8>> {
        match input {
            AudioProcessorOutputEnum::EqBins(bins) => {
                Some(self.eq_painter.paint(bins))
            }
            AudioProcessorOutputEnum::NoteInfo(note_info_option) => {
                // note_info is optional because the pitch detector is strict
                if let Some(note_info) = note_info_option {
                    Some(self.tuner_painter.paint(note_info))
                }
                else {
                    None
                }
            }
        }
    }
}