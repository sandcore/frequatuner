use rustfft::{FftPlanner, num_complex::Complex};

use crate::LEDS_MAX_Y;

/* 
An audio buffer is taken, an FFT transform is done, the FFT is distributed over a fixed number of bins.

Because first and last bin are full but not valid signal, the resultbins gets 2 extra which get removed on output.

AudioFrequalizer runs the analysis process and has the output for visual processing.
*/

pub struct AudioFrequalizer {
    fft_planner: FftPlanner<f32>,
    samples_buffer: Vec<f32>,
    samples_max: usize,
    res_edges: AdaptiveResultEdges, // the edges for the bins the FFT result gets placed in, keep state so range can be adapted
    pub eq_bins: Vec<f32>
}

// Number of samples and sample rate determine the min and max frequencies that are measured by the FFT
impl AudioFrequalizer {
    pub fn new(num_bins: usize, sample_rate: u32) -> Self {
        let samples_max = 2048; // 2048 was about the max I could fill the FFT transform with before crashing the ESP and it gives a good range of frequencies, slightly more than can be heard by most humans.
        let eq_bins = Vec::with_capacity(num_bins as usize);

        // set up the result bins, need to init with edges
        let mut res_edges = AdaptiveResultEdges::new(samples_max, num_bins, sample_rate);
        res_edges.create_log_bin_edges();

        AudioFrequalizer {
            fft_planner: FftPlanner::new(),
            samples_buffer: vec![],
            samples_max,
            res_edges,
            eq_bins
        }
    }

    pub fn frequalize(&mut self, mut samples: Vec<f32>) {
        self.samples_buffer.append(&mut samples);

        while self.samples_buffer.len() >= self.samples_max as usize { 
            let samples_to_process: Vec<f32> = self.samples_buffer.splice(0..self.samples_max, []).collect();
    
            // do the sequential FFT processing steps
            let raw_buffer = RawBuffer::new(samples_to_process);
            let fft_over_samples = raw_buffer.fft_transform(&mut self.fft_planner);
            let adapted_edges = fft_over_samples.adapt_edges(&mut self.res_edges);
            let fft_result_bins = adapted_edges.distribute_fft_to_fixed_bins(&mut self.res_edges);
            let normalized_fft_result_bins = fft_result_bins.normalize_logarithmic_bins();
            self.eq_bins = normalized_fft_result_bins.output();
        }
    }
}

// raw buffer of samples -> filter -> fft applied over samples -> adap adaptive bin edges -> fft distributed to result bins -> fft in result bins normalized. Every state change destroys prev state.
pub struct RawBuffer{
    samples: Vec<f32>,
}

struct FFTOverSamples {
    ffted_samples: Vec<Complex<f32>>
}

struct AdaptedEdges {
    ffted_samples: Vec<Complex<f32>>
}

struct FFTResultBins{
    bins: Vec<f32>
}
struct NormalizedFFTResultBins{
    bins: Vec<f32>
}

impl RawBuffer {
    pub fn new(samples: Vec<f32>) -> RawBuffer {
        RawBuffer {
            samples,
        }
    }
    
    fn fft_transform(self, fft_planner: &mut FftPlanner<f32>) -> FFTOverSamples {
        // FFT planner needs Complex values
        let mut output_vec: Vec<Complex<f32>> = self.samples.into_iter().map(|x| Complex::new(x, 0.0)).collect();

        let fft = fft_planner.plan_fft_forward(output_vec.len());
        fft.process(&mut output_vec);

        FFTOverSamples{ffted_samples: output_vec}
    }

}

impl FFTOverSamples {
    fn adapt_edges(self, res_edges: &mut AdaptiveResultEdges) -> AdaptedEdges {
        // adapt edges slowly to the min and max range of peaks in the latest signal
        let fft_len = self.ffted_samples.len();
        let freq_resolution = res_edges.sample_rate as f32/ fft_len as f32;
        let mut min_freq = res_edges.absolute_min_freq;
        let mut max_freq = res_edges.absolute_max_freq;

        for (i, &complex) in self.ffted_samples.iter().enumerate().take(fft_len/2 - 1) {
            let mag = complex.norm(); // normalize the magnitudes.
            if mag > 0.0 {
                min_freq = i as f32 * freq_resolution;
                break;
            }
        }

        for (i, &complex) in self.ffted_samples.iter().enumerate().take(fft_len/2 - 1).rev() {
            let mag = complex.norm(); // normalize the magnitudes.
            if mag > 0.0 {
                max_freq =  i as f32 * freq_resolution;
                break;
            }
        }

        // Gradually adapt range
        /*let adaptation_rate = 0.1;
        res_edges.current_min_freq += (min_freq - res_edges.current_min_freq) * adaptation_rate;
        res_edges.current_max_freq += (max_freq - res_edges.current_max_freq) * adaptation_rate;

        // Ensure current frequencies stay within absolute limits
        res_edges.current_min_freq = res_edges.current_min_freq
            .max(res_edges.absolute_min_freq)
            .min(res_edges.absolute_max_freq);
        res_edges.current_max_freq = res_edges.current_max_freq
            .max(res_edges.absolute_min_freq)
            .min(res_edges.absolute_max_freq);

        res_edges.create_log_bin_edges();*/

        AdaptedEdges {
            ffted_samples: self.ffted_samples
        }
    }
}
impl AdaptedEdges{
    fn distribute_fft_to_fixed_bins(self, res_edges: &mut AdaptiveResultEdges) -> FFTResultBins {
        //println!("{:?}", res_bins.bins);
        let mut result = vec![0.0; res_edges.edges.len() - 1];
        let fft_len = self.ffted_samples.len();
        let freq_resolution = res_edges.sample_rate as f32/ fft_len as f32;
    
        for (i, &complex) in self.ffted_samples.iter().enumerate().take(fft_len/2 - 1) {
            let mag = complex.norm(); // normalize the magnitudes.
            let freq = i as f32 * freq_resolution;
            //println!("{} {} {}", i, freq, fft_result[i].norm());
    
            let bin_index = if freq < res_edges.edges[0] {
                0 // Put frequencies below min_freq in the first bin
            } else if freq > res_edges.edges[res_edges.edges.len() - 1] {
                res_edges.edges.len() - 2 // Put frequencies above max_freq in the last bin
            } else {
                res_edges.edges.partition_point(|&x| x < freq).saturating_sub(1)
            };
    
            result[bin_index] += mag;
        }

        for i in 0..result.len() {
            let bin_width = res_edges.edges[i + 1] - res_edges.edges[i];
            if bin_width > 0.0 {
                result[i] /= bin_width; // bigger bins get bigger values, normalize that
            }
        }

        FFTResultBins{
            bins: result
        }
    }
}

impl FFTResultBins {
    // find max magnitude and normalize the values with that max
    fn normalize_logarithmic_bins(mut self) -> NormalizedFFTResultBins {
        let max_magnitude = self.bins.iter().cloned().fold(0.0_f32, f32::max);

        if max_magnitude > 0.0 {
            for bin in self.bins.iter_mut() {
                *bin /= max_magnitude;
            }
        }

        NormalizedFFTResultBins{
            bins: self.bins
        }
    }
}

impl NormalizedFFTResultBins {
    fn output(self) -> Vec<f32> {
        //remove first and last bin
        self.bins[1..(self.bins.len()-1)].to_vec()
    }
}


// Adaptive result bins: based on the frequency ranges adjust the bins slowly to the measured range.
struct AdaptiveResultEdges {
    edges: Vec<f32>, // edges for the bins, 1 more edge than number of bins. Based on these edges the FFT is placed in bins. Every edge is a frequency value.
    num_bins: usize,
    sample_rate: u32,
    absolute_min_freq: f32,
    absolute_max_freq: f32,
    current_min_freq: f32,
    current_max_freq: f32,
}
impl AdaptiveResultEdges {
    fn new(num_samples: usize, num_bins: usize, sample_rate: u32) -> AdaptiveResultEdges {
        //set up the edges for the bins. 
        let min_freq = (sample_rate as f32 / num_samples as f32).max(30.0); // Use 30 hz or the lowest possibly measured freq value, whichever is higher
        let max_freq = (sample_rate as f32 / 2.0).min(18000.0); // Use 18000 Hz or Nyquist frequency (samp rate/2), whichever is lower

        AdaptiveResultEdges {
            edges: vec![],
            num_bins,
            sample_rate,
            absolute_min_freq: min_freq,
            absolute_max_freq: max_freq,
            current_min_freq: min_freq,
            current_max_freq: max_freq,
        }
    }

    fn create_log_bin_edges(&mut self) {
        let num_edges = self.num_bins + 2 + 1; // Because first and last bin are getting filled up with some (electrical?) noise. The two extra get removed on outputs

        let mut edges = Vec::with_capacity(num_edges as usize);
        for i in 0..=num_edges {
            let t = i as f32 / num_edges as f32;
            let freq = self.current_min_freq * (self.current_max_freq / self.current_min_freq).powf(t);
            edges.push(freq);
        }
        self.edges = edges;
    }
}