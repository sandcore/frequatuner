use std::collections::VecDeque;

use pitch_detection::detector::{mcleod::McLeodDetector, PitchDetector};
use pitch_detector::note::NoteDetectionResult;
use fundsp::hacker32::*;

// Every 1024 samples a pitch detection loop is started. The DSP filter (low and highpass) is configured at a 64 sample buffer. Probably a good idea to keep the amount of samples used in a
// pitch detection loop a multiple of the DSP sample buffer when changes values around.

/*
GiTuner runs the pitch detection process and supplies output.
*/
pub struct GiTuner {
    samples: RawBuffer,
    samples_max: usize,

    pitch_detector: McLeodDetector<f32>,

    // state info
    recent_freqs: VecDeque<f32>,

    // output for comm with visual processor
    pub note_info: Option<NoteDetectionResult> //mcleoddetector doesnt always find a result. In that case output None
}

impl GiTuner {
    pub fn new(buffer_length: usize) -> Self {
        // better frequency detection resolution by using more (1024+) samples, otherwise low E string is not measured right
        // https://www.cycfi.com/2018/04/fast-and-efficient-pitch-detection-bliss/
        let samples_max_analysis = 2048; // number of samples in every analysis run.

        GiTuner {
            samples: RawBuffer::new(buffer_length),
            samples_max: samples_max_analysis, // matches the input to pitch_detector below otherwise that errors out
            pitch_detector: McLeodDetector::new(samples_max_analysis, samples_max_analysis/2),
            recent_freqs: VecDeque::new(),
            note_info: None 
        }
    }

    pub fn tune(&mut self, sample: f32, sample_rate: u32) {
        self.samples.buffer.push(sample);

        if self.samples.buffer.len() >= self.samples_max {
            let applied_filters = self.samples.apply_filters();
            let pitch_detected = applied_filters.pitch_detection(&mut self.pitch_detector, &mut self.recent_freqs, sample_rate);
            self.note_info = pitch_detected.note_info();
            
            self.samples.buffer.clear();
        }
    }
}

/*
Consecutive processing steps, used typestate pattern here to keep some overview
 - raw buffer (not destroyed after state change, used for further sample collection)
 - low/highpass filters applied to clean up the audio samples a bit
 - get pitch. Pitch is from pitch_detectION crate but we get the note from the pitch_detecTOR crate later. Pitch from pitch_detection crate because that gave better results for my situation. 
 - process mean (so tuner jumps around less) and get note info from pitch_detector
*/

struct RawBuffer {
    buffer: Vec<f32>,

    // Filter and related settings
    lowpass_filter: An<FixedSvf<f32, LowpassMode<f32>>>,
    highpass_filter: An<FixedSvf<f32, HighpassMode<f32>>>,
}
impl RawBuffer {
    pub fn new(buffer_length: usize) -> RawBuffer {
        // fundsp filters. 
        // pre-amp lower freqs a little because low E string is hard to detect
        // lowpass and highpass are combined into a composite type
        // need to be persisted because fundsp filters work by mainining internal state
        // another reason RawBuffer can't be dropped

        let low_freq = 30.0; //low cutoff frequency for highpass
        let high_freq = 18000.0; // high cutoff frequency for lowpass
        let q = 0.707; // "Q factor", 0.707 is supposed to be a good / safe value (:

        let lowpass_filter= lowpass_hz(high_freq, q);
        let highpass_filter = highpass_hz(low_freq, q);

        RawBuffer {
            buffer: Vec::with_capacity(buffer_length),
            lowpass_filter,
            highpass_filter
        }
    }
    fn apply_filters(&mut self) -> FiltersApplied {
        let max_dsp_buffer = 64; // max size of the processing used by fundsp
        let max_dsp_buffer_idx = 63; // for use in index calculations

        let gain = 2.0f32; // Boost signal because of low amplitude direct guitar input

        // Used BufferArrays before integrating all parts of EqTuner project together, and that worked before. But in the integrated project
        // BufferArrays cause a panic without a source location, guess stack related.
        let mut dsp_buff = BufferVec::new(1);
        let mut dsp_lowpassed_values = BufferVec::new(1);
        let mut dsp_highpassed_values = BufferVec::new(1);
        
        let mut output_vec = vec![0f32; self.buffer.len()];
        let mut dspbuffer_counter = 0;

        for (i, sample) in self.buffer.iter_mut().enumerate() {
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

        FiltersApplied {
            filtered_buffer: output_vec
        }
    }
}
struct FiltersApplied {
    filtered_buffer: Vec<f32>
}
impl FiltersApplied {
    pub fn pitch_detection(self, detector: &mut McLeodDetector<f32>, recent_freqs: &mut VecDeque<f32>, sample_rate: u32) -> PitchDetermined {
        let mut mean_freq = None; // by default we have no new mean frequency

        if let Some(pitch) = detector.get_pitch(&self.filtered_buffer, sample_rate as usize, 0.00005, 0.5) {
            let frequency = pitch.frequency;
            recent_freqs.push_back(frequency);
        }

        if recent_freqs.len() == 2 { // a new frequency was pushed in the vecdeque, determine new mean 
            mean_freq = Some(recent_freqs.iter().sum::<f32>() / recent_freqs.len() as f32);
            recent_freqs.pop_front();
        }

        PitchDetermined {
            mean_freq
        }
    }
}
struct PitchDetermined {
    mean_freq: Option<f32>
}
impl PitchDetermined {
    /*
    INFO FROM PITCH DETECTOR CRATE:
            
    pub struct NoteDetectionResult 
        /// The predominant frequency detected from a signal.
        pub actual_freq: f64,

        /// The note name of the detected note.
        pub note_name: NoteName,

        /// The expected frequency of the detected note.
        pub note_freq: f64,

        /// The octave of the detected note.
        pub octave: i32,

        /// The degree to which the detected not is in tune, expressed in cents. The absolute maximum `cents_offset` is
        /// 50, since anything larger than 50 would be considered the next or previous note.
        pub cents_offset: f64,

        /// The note name of the note that comes before the detected note. Not commonly used.
        pub previous_note_name: NoteName,

        /// The note name of the note that comes after the detected note. Not commonly used.
        pub next_note_name: NoteName,

        /// A `NoteDetectionResult` will be marked as `in_tune` if the `cents_offset` is less than
        /// [`MAX_CENTS_OFFSET`](crate::core::constants::MAX_CENTS_OFFSET).
        pub in_tune: bool,
    */ 
    pub fn note_info(self) -> Option<NoteDetectionResult> {
        if let Some(mean_freq) = self.mean_freq {
            if let Ok(res) = NoteDetectionResult::try_from(mean_freq as f64) {
                return Some(res);
            }
        }  
        None // we don't have a new mean or detection result, don't update the tuner output
    }
}