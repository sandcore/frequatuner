
use pitch_detector::note::NoteDetectionResult;
use fundsp::hacker32::*;

use crate::{EqTunerModeEnum, LEDS_MAX_Y};

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
    tuner: audio_tuner::GiTuner,

    lowpass_filter: An<FixedSvf<f32, LowpassMode<f32>>>,
    highpass_filter: An<FixedSvf<f32, HighpassMode<f32>>>,
}
impl AudioProcessor {
    pub fn new(sample_rate: &u32) -> Self {
        // fundsp filters as a pre-processor. Removes a lot of audio glitching when there isn't a lot coming in.
        // lowpass and highpass need to be persisted because fundsp filters work by mainining internal state

        let low_freq = 30.0; //low cutoff frequency for highpass
        let high_freq = 18000.0; // high cutoff frequency for lowpass
        let q = 0.707; // "Q factor", 0.707 is supposed to be a good / safe value (:

        let lowpass_filter= lowpass_hz(high_freq, q);
        let highpass_filter = highpass_hz(low_freq, q);

        AudioProcessor {
            lowpass_filter,
            highpass_filter,
            sample_rate: *sample_rate,
            frequalizer: audio_fft_binner::AudioFrequalizer::new(LEDS_MAX_Y, *sample_rate),
            tuner: audio_tuner::GiTuner::new()
        }
    }

    pub fn process(&mut self, audio_values: Vec<f32>, mode: &EqTunerModeEnum) {
        let lowhighpass_audio_vals = self.apply_lowhighpass(audio_values);
        match mode {
            EqTunerModeEnum::Equalizer => {
                self.frequalizer.frequalize(lowhighpass_audio_vals)
            },
            EqTunerModeEnum::Tuner => {
                self.tuner.tune(lowhighpass_audio_vals, self.sample_rate)
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

    fn apply_lowhighpass(&mut self, mut samples: Vec<f32>) -> Vec<f32> {
        let max_dsp_buffer = 64; // max size of the processing used by fundsp
        let max_dsp_buffer_idx = 63; // for use in index calculations

        let gain = 3.0f32; // Boost signal because of low amplitude on ADC I2S module. 

        let mut dsp_buff = BufferVec::new(1);
        let mut dsp_lowpassed_values = BufferVec::new(1);
        let mut dsp_highpassed_values = BufferVec::new(1);
        
        let mut output_vec = vec![0f32; samples.len()];
        let mut dspbuffer_counter = 0;

        for (i, sample) in samples.iter_mut().enumerate() {
            let gained_sample = *sample * gain;
            dsp_buff.buffer_mut().set_f32(0, dspbuffer_counter, gained_sample);

            if dspbuffer_counter == max_dsp_buffer_idx {
                self.lowpass_filter.process(max_dsp_buffer, &dsp_buff.buffer_ref(), &mut dsp_lowpassed_values.buffer_mut());
                self.highpass_filter.process(max_dsp_buffer, &dsp_lowpassed_values.buffer_ref(), &mut dsp_highpassed_values.buffer_mut());

                // copy filtered values in the samples from the current index
                output_vec[(i-max_dsp_buffer_idx)..=i].copy_from_slice(dsp_highpassed_values.buffer_mut().channel_f32(0));

                dspbuffer_counter = 0;
            }
            else {
                dspbuffer_counter += 1;
            }
        }
        output_vec
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

    pub fn process_and_output(&mut self, input: AudioProcessorOutputEnum) -> Option<Vec<u8>> {
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