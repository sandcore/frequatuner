use std::collections::VecDeque;

use pitch_detection::detector::{mcleod::McLeodDetector, PitchDetector};
use pitch_detector::note::NoteDetectionResult;

// Every X samples a pitch detection loop is started. The DSP filter (low and highpass) uses a 64 sample buffer. Probably a good idea to keep the amount of samples used in a
// pitch detection loop a multiple of the DSP sample buffer when changes values around.

/*
GiTuner runs the pitch detection process and supplies output.
*/
pub struct GiTuner {
    samples_buffer: Vec<f32>,
    samples_max: usize,

    pitch_detector: McLeodDetector<f32>,

    // state info
    recent_freqs: VecDeque<f32>,

    // output for comm with visual processor
    pub note_info: Option<NoteDetectionResult> //mcleoddetector doesnt always find a result. In that case output None
}

impl GiTuner {
    pub fn new() -> Self {
        // There is better frequency detection resolution by using more (1024+) samples, otherwise low E string is not measured right
        // https://www.cycfi.com/2018/04/fast-and-efficient-pitch-detection-bliss/
        let samples_max_analysis = 2048; // number of samples in every analysis run.

        GiTuner {
            samples_buffer: vec![],
            samples_max: samples_max_analysis, // matches the input to pitch_detector below otherwise that errors out
            pitch_detector: McLeodDetector::new(samples_max_analysis, samples_max_analysis/2),
            recent_freqs: VecDeque::new(),
            note_info: None 
        }
    }

    pub fn tune(&mut self, mut samples: Vec<f32>, sample_rate: u32) {
        self.samples_buffer.append(&mut samples);

        while self.samples_buffer.len() >= self.samples_max as usize { 
            let samples_to_process: Vec<f32> = self.samples_buffer.splice(0..self.samples_max, []).collect();

            let raw_buffer = RawBuffer::new(samples_to_process);
            let pitch_detected = raw_buffer.pitch_detection(&mut self.pitch_detector, &mut self.recent_freqs, sample_rate);
            self.note_info = pitch_detected.note_info();
        }
    }
}

/*
Consecutive processing steps, used typestate pattern
 - raw buffer
 - get pitch. Pitch is from pitch_detectION crate but we get the note from the pitch_detecTOR crate later. Pitch from pitch_detection crate because that gave better results for my situation. 
 - process mean (so tuner jumps around less) and get note info from pitch_detector
*/

struct RawBuffer {
    buffer: Vec<f32>,
}
impl RawBuffer {
    pub fn new(samples: Vec<f32>) -> RawBuffer {
        RawBuffer {
            buffer: samples,
        }
    }

    pub fn pitch_detection(self, detector: &mut McLeodDetector<f32>, recent_freqs: &mut VecDeque<f32>, sample_rate: u32) -> PitchDetermined {
        let mut mean_freq = None; // by default we have no new mean frequency

        if let Some(pitch) = detector.get_pitch(&self.buffer, sample_rate as usize, 0.00005, 0.5) {
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