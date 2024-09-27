use rustfft::{FftPlanner, num_complex::Complex};
use fundsp::hacker32::*;

/* 
Originally this file just called functions sequentially until all the processing steps of the audio_chunks were done.

When I came back 1.5 months later I initially had no clue anymore what was happening on revisit. 
Refactored (typestate pattern), to make the code speak more and make the next revisit less painful.

The main idea is that an audio buffer is taken, an FFT transform is done, the FFT is distributed over a fixed number of bins.

AudioFrequalizer runs the analysis process and has the output for visual processing.
*/

pub struct AudioFrequalizer {
    num_bins: usize,
    fft_planner: FftPlanner<f32>,
    samples_buffer: Vec<f32>,
    samples_max: u32,
    pub eq_bins: Vec<f32>
}

impl AudioFrequalizer {
    pub fn new(num_bins: usize) -> Self {
        let samples_max = 2048; // 2048 was about the max I could fill the FFT transform with before crashing the ESP and it gives a good range of frequencies, slightly more than can be heard by most humans.
        let eq_bins = Vec::with_capacity(num_bins as usize);

        AudioFrequalizer {
            num_bins,
            fft_planner: FftPlanner::new(),
            samples_buffer: vec![],
            samples_max,
            eq_bins
        }
    }

    // The caller chooses how many frequency bins are desired. A bigger led matrix will have room to display more bins than a smaller.
    // Number of samples and sample rate determine the min and max frequencies that are measured by the FFT
    pub fn frequalize(&mut self, mut samples: Vec<f32>, sample_rate: u32) {
        self.samples_buffer.append(&mut samples);

        while self.samples_buffer.len() >= self.samples_max as usize { 
            let samples_to_process: Vec<f32> = self.samples_buffer.splice(0..self.samples_buffer.len(), []).collect();
            let num_samples = samples_to_process.len();

            // set up the result bins
            let res_bins = ResultBins::new(num_samples, self.num_bins, sample_rate);
    
            // do the sequential FFT processing steps
            let raw_buffer = RawBuffer::new(samples_to_process);
            let mut filters_applied = raw_buffer.apply_filters();
            let fft_over_samples = filters_applied.fft_transform(&mut self.fft_planner);
            let fft_result_bins = fft_over_samples.distribute_fft_to_fixed_bins(res_bins);
            let normalized_fft_result_bins = fft_result_bins.normalize_logarithmic_bins();
            self.eq_bins = normalized_fft_result_bins.fft_resultbins.bins;
        }
    }
}

// raw buffer of samples -> filter -> fft applied over samples -> fft distributed to result bins -> fft in result bins normalized. Every state change destroys prev state.

pub struct RawBuffer{
    samples: Vec<f32>,

    lowpass_filter: An<FixedSvf<f32, LowpassMode<f32>>>,
    highpass_filter: An<FixedSvf<f32, HighpassMode<f32>>>,
}

struct FiltersApplied {
    filtered_buffer: Vec<Complex<f32>>
}

struct FFTOverSamples<'a> {
    ffted_samples: &'a mut Vec<Complex<f32>>
}
struct FFTResultBins{
    fft_resultbins: ResultBins
}
struct NormalizedFFTResultBins{
    fft_resultbins: ResultBins
}

impl RawBuffer {
    pub fn new(samples: Vec<f32>) -> RawBuffer {
        // fundsp filters. 
        // lowpass and highpass are combined into a composite type
        // need to be persisted because fundsp filters work by mainining internal state

        let low_freq = 40.0; //low cutoff frequency for highpass
        let high_freq = 17000.0; // high cutoff frequency for lowpass
        let q = 0.707; // "Q factor", 0.707 is supposed to be a good / safe value (:

        let lowpass_filter= lowpass_hz(high_freq, q);
        let highpass_filter = highpass_hz(low_freq, q);

        RawBuffer {
            samples,
            lowpass_filter,
            highpass_filter
        }
    }

    fn apply_filters(mut self) -> FiltersApplied {
        let max_dsp_buffer = 64; // max size of the processing used by fundsp
        let max_dsp_buffer_idx = 63; // for use in index calculations

        let gain = 2.0f32; // Boost signal because of low amplitude direct guitar input. Second gain on top of the gain in the main processor.

        // Used BufferArrays before integrating all parts of EqTuner project together, and that worked before. But in the integrated project
        // BufferArrays cause a panic without a source location, guess stack related.
        let mut dsp_buff = BufferVec::new(1);
        let mut dsp_lowpassed_values = BufferVec::new(1);
        let mut dsp_highpassed_values = BufferVec::new(1);
        
        let mut output_vec = vec![0f32; self.samples.len()];
        let mut dspbuffer_counter = 0;

        for (i, sample) in self.samples.iter_mut().enumerate() {
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
        let complex_output_vec = output_vec.into_iter().map(|x| Complex::new(x, 0.0)).collect();

        FiltersApplied {
            filtered_buffer: complex_output_vec
        }
    }
}

impl FiltersApplied {
    fn fft_transform(&mut self, fft_planner: &mut FftPlanner<f32>) -> FFTOverSamples {
        let fft = fft_planner.plan_fft_forward(self.filtered_buffer.len());
        fft.process(&mut self.filtered_buffer);

        FFTOverSamples{ffted_samples: &mut self.filtered_buffer}
    }
}

impl FFTOverSamples<'_> {
    fn distribute_fft_to_fixed_bins(self, mut res_bins: ResultBins) -> FFTResultBins {
        //println!("{:?}", res_bins.bins);
        let mut result = vec![0.0; res_bins.edges.len() - 1];
        let fft_len = self.ffted_samples.len();
        let freq_resolution = res_bins.sample_rate as f32/ fft_len as f32;
    
        for (i, &complex) in self.ffted_samples.iter().enumerate().take(fft_len/2 - 1) {
            let mag = complex.norm(); // normalize the magnitudes.
            let freq = i as f32 * freq_resolution;
            //println!("{} {} {}", i, freq, fft_result[i].norm());
    
            let bin_index = if freq < res_bins.edges[0] {
                0 // Put frequencies below min_freq in the first bin
            } else if freq > res_bins.edges[res_bins.edges.len() - 1] {
                res_bins.edges.len() - 2 // Put frequencies above max_freq in the last bin
            } else {
                res_bins.edges.partition_point(|&x| x < freq).saturating_sub(1)
            };
    
            result[bin_index] += mag;
        }

        for i in 0..result.len() {
            let bin_width = res_bins.edges[i + 1] - res_bins.edges[i];
            if bin_width > 0.0 {
                result[i] /= bin_width; // bigger bins get bigger values, normalize that
            }
        }
        res_bins.bins = result;

        FFTResultBins{fft_resultbins:res_bins}
    }
}
impl FFTResultBins {
    // find max magnitude and normalize the values with that max
    fn normalize_logarithmic_bins(mut self) -> NormalizedFFTResultBins {
        let max_magnitude = self.fft_resultbins.bins.iter().cloned().fold(0.0_f32, f32::max);

        if max_magnitude > 0.0 {
            for bin in self.fft_resultbins.bins.iter_mut() {
                *bin /= max_magnitude;
            }
        }
        NormalizedFFTResultBins{fft_resultbins: self.fft_resultbins}
    }
}
struct ResultBins {
    edges: Vec<f32>, // edges for the bins, 1 more edge than number of buns. Based on these edges the FFT is placed in bins. Every edge is a frequency value.
    bins: Vec<f32>,
    sample_rate: u32
}
impl ResultBins {
    fn new(num_samples: usize, num_bins: usize, sample_rate: u32) -> ResultBins {
        //set up the edges for the bins
        let min_freq = (sample_rate as f32 / num_samples as f32).max(70.0); // Use 35 hz or the lowest possibly measured freq value, whichever is higher
        let max_freq = (sample_rate as f32 / 2.0).min(1500.0); // Use 18000 Hz or Nyquist frequency (samp rate/2), whichever is lower

        let mut edges = Vec::with_capacity(num_bins as usize + 1);
        for i in 0..=num_bins {
            let t = i as f32 / num_bins as f32;
            let freq = min_freq * (max_freq / min_freq).powf(t);
            edges.push(freq);
        }

        ResultBins {
            edges,
            bins: vec![],
            sample_rate
        }
    }
}