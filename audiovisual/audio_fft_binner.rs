use rustfft::{FftPlanner, num_complex::Complex};

/* 
Originally this file just called functions sequentially until all the processing steps of the audio_chunks were done.

When I came back 1.5 months later I initially had no clue anymore what was happening on revisit. 
Refactored (typestate pattern), to make the code speak more and make the next revisit less painful.

The main idea is that an audio buffer is taken, an FFT transform is done, the FFT is distributed over a fixed number of bins.

AudioFrequalizer runs the analysis process and has the output for visual processing.
*/

pub struct AudioFrequalizer {
    num_bins: u8,
    fft_planner: FftPlanner<f32>,
    samples_buffer: RawBuffer,
    samples_max: u32,
    pub eq_bins: Vec<f32>
}

impl AudioFrequalizer {
    pub fn new(num_bins: u8) -> Self {
        let samples_max = 2048; // 2048 was about the max I could fill the FFT transform with before crashing the ESP and it gives a good range of frequencies, slightly more than can be heard by most humans.
        let eq_bins = Vec::with_capacity(num_bins as usize);

        AudioFrequalizer {
            num_bins,
            fft_planner: FftPlanner::new(),
            samples_buffer: RawBuffer{samples:vec![]},
            samples_max,
            eq_bins
        }
    }

    // The caller chooses how many frequency bins are desired. A bigger led matrix will have room to display more bins than a smaller.
    // Number of samples and sample rate determine the min and max frequencies that are measured by the FFT
    pub fn frequalize(&mut self, sample: f32, sample_rate: u32) {
        self.samples_buffer.samples.push(Complex{re: sample, im: 0.0f32});

        if self.samples_buffer.samples.len() >= self.samples_max as usize { 
            let num_samples = self.samples_buffer.samples.len();

            // set up the result bins
            let res_bins = ResultBins::new(num_samples, self.num_bins, sample_rate);
    
            // do the sequential FFT processing steps
            let fft_over_samples = self.samples_buffer.fft_transform(&mut self.fft_planner);
            let fft_result_bins = fft_over_samples.distribute_fft_to_fixed_bins(res_bins);
            let normalized_fft_result_bins = fft_result_bins.normalize_logarithmic_bins();
            self.eq_bins = normalized_fft_result_bins.fft_resultbins.bins;

            //println!("{:?}", self.eq_bins);
            self.samples_buffer.samples.clear();
        }
    }
}

// raw buffer of samples -> fft applied over samples -> fft distributed to result bins -> fft in result bins normalized
// after first state change the raw buffer does NOT get dropped, it is used in the frequalizer as the samples buffer and filled again with new samples

pub struct RawBuffer{
    pub samples: Vec<Complex<f32>>
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
    fn fft_transform(&mut self, fft_planner: &mut FftPlanner<f32>) -> FFTOverSamples {
        let fft = fft_planner.plan_fft_forward(self.samples.len());
        fft.process(&mut self.samples);

        FFTOverSamples{ffted_samples: &mut self.samples}
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
    fn new(num_samples: usize, num_bins: u8, sample_rate: u32) -> ResultBins {
        //set up the edges for the bins
        let min_freq = (sample_rate as f32 / num_samples as f32).max(35.0); // Use 35 hz or the lowest possibly measured freq value, whichever is higher
        let max_freq = (sample_rate as f32 / 2.0).min(18000.0); // Use 18000 Hz or Nyquist frequency (samp rate/2), whichever is lower

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