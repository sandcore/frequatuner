use std::sync::atomic::{AtomicBool, Ordering}; // for the interrupt handling
use std::time::{Duration, SystemTime};

use esp_idf_hal::{delay::FreeRtos, gpio::{PinDriver, AnyIOPin, Input}, i2s::{I2sDriver, I2sRx}};
use ws2812_esp32_rmt_driver::driver::Ws2812Esp32RmtDriver;

mod esp32s3_hw; // driver wrappers for confirmed working on-board and connected hardware in my setup
use esp32s3_hw::Esp32S3c1;

mod audiovisual; // process audio feed and output to led matrix

pub enum EqTunerMode {
    Equalizer,
    Tuner
}

/*
EqTuner keeps track of if we're in equalizer or tuner mode, contains the drivers that are used and is a centralized spot for 
initializing some informational variables relating to the specific project / hardware used.

Current logic is only made for a situation where the number of frequency bins in eq mode is equal to the number of rows in the ledmatrix (I'm 
using vertically placed 8x32 matrix, so 32 freq bins). To change that some minor changes in the frequency visualizer are necessary.
*/
struct EqTuner<'a> {
    mode: EqTunerMode,
    audio_driver: I2sDriver<'a, I2sRx>,
    ledmatrix_driver: Ws2812Esp32RmtDriver,
    mode_button_driver: PinDriver<'a, AnyIOPin, Input>,

    sample_rate: u32,
    num_frequency_bins: u8,
    frame_duration: Duration, // ledmatrix starts glitching if this is too short 
    last_visual_update: SystemTime,
    ledmatrix_max_x: usize,
    ledmatrix_max_y: usize
}

impl <'a>EqTuner<'a> {
    fn new(sample_rate:u32, num_frequency_bins: u8, ledmatrix_max_x: usize, ledmatrix_max_y: usize) -> Self {
        let mut esp32 = Esp32S3c1::new();

        let audio_driver = esp32s3_hw::get_linejack_i2s_driver(&mut esp32, sample_rate, 0, 5, 7, 6);        
        let ledmatrix_driver = esp32s3_hw::get_ws2812ledstrip_driver(&mut esp32, 3, 18);
        
        // set up the mode switch button and set an interrupt on it
        let mut mode_button_driver = esp32s3_hw::get_on_board_boot_button(&mut esp32, None);
        mode_button_driver.set_interrupt_type(esp_idf_hal::gpio::InterruptType::NegEdge).ok();
        unsafe {
            mode_button_driver.subscribe(boot_button_callback).expect("Interrupt subscribe failed");
        }
        mode_button_driver.enable_interrupt().ok();

        EqTuner {
            mode: EqTunerMode::Equalizer,
            audio_driver,
            ledmatrix_driver,
            mode_button_driver,
            sample_rate,
            num_frequency_bins,
            frame_duration: Duration::from_micros(60000), // 16 fps is more than enough. Won't be exact due to execution times but should be a fast enough constant refresh rate that doesnt glitch the matrix
            last_visual_update: SystemTime::now(),
            ledmatrix_max_x,
            ledmatrix_max_y
        }
    }

    fn check_mode_switch(&mut self) {
        if BOOTTON_PRESSED.load(Ordering::Relaxed) {
            BOOTTON_PRESSED.store(false, Ordering::Relaxed);
            self.switch_mode(None);
            self.mode_button_driver.enable_interrupt().ok();
        }
    }

    fn switch_mode(&mut self, desired_mode: Option<EqTunerMode>) {
        match desired_mode {
            Some(EqTunerMode::Equalizer) => self.mode = EqTunerMode::Equalizer,
            Some(EqTunerMode::Tuner) => self.mode = EqTunerMode::Tuner,
            None => {
                match self.mode {
                    EqTunerMode::Equalizer => self.mode = EqTunerMode::Tuner,
                    EqTunerMode::Tuner => self.mode = EqTunerMode::Equalizer
                }
            }
        }
        self.display_switch_screen();
    }

    fn display_ledmatrix(&mut self, colors: &Vec<u8>) {
        let now = SystemTime::now();
        let elapsed = now.duration_since(self.last_visual_update).unwrap_or(Duration::ZERO);

        if elapsed >= self.frame_duration {
            self.ledmatrix_driver.write(colors).ok();
            self.last_visual_update = now;
        }
    }

    fn display_switch_screen(&mut self) {
        let graphic = vec![4u8; self.ledmatrix_max_x*self.ledmatrix_max_y*3];
        
        match self.mode {
            EqTunerMode::Equalizer => {

            },
            EqTunerMode::Tuner => {

            }
        }

        self.ledmatrix_driver.write(&graphic).ok();
        FreeRtos::delay_ms(1000) // bask in the glory of the switch screen
    }
}

static BOOTTON_PRESSED: AtomicBool = AtomicBool::new(false);

fn boot_button_callback() {
    BOOTTON_PRESSED.store(true, Ordering::Relaxed);
}

fn main() {
    esp_idf_hal::sys::link_patches();
    let mut eq_tuner = EqTuner::new(48000, 32, 8, 32);
    let mut audiobuffer = [0; 3072]; // buffer for the sound driver

    // working with an 8x32 led matrix. The equalizer will display a frequency range on every row
    let mut audio_processor = audiovisual::AudioProcessor::new(audiobuffer.len(), eq_tuner.num_frequency_bins, eq_tuner.sample_rate);
    let mut visual_processor = audiovisual::VisualProcessor::new(eq_tuner.ledmatrix_max_x, eq_tuner.ledmatrix_max_y);

    /*
    Main loop: read the audiobuffer from the i2s driver and run
    the audio processor on it. 

    The visual processor reads audioprocessor output, processes, and outputs the color array (size is ledmatrix_x*ledmatrix_y*3 for g,r,b on every led) 
    */
    loop { 
        FreeRtos::delay_ms(1); // give OS a chance to do some threading
        
        // Switch from equalizer to tuner and back on button presses
        eq_tuner.check_mode_switch();

        let bytes_read: usize = eq_tuner.audio_driver.read(&mut audiobuffer, 1000).unwrap();
        for chunks in audiobuffer.chunks(4).take(bytes_read / 4) { 
            // on Esp32S3 for my two devices the MEMS microphone outputted the middle two bytes and garbage in the 1st and 4th. The linejack hardware outputs all 4 useful bytes. Currently working with linejack
            let unprocessed_audio_value = i32::from_le_bytes( [chunks[0], chunks[1], chunks[2], chunks[3]]);
            audio_processor.process(unprocessed_audio_value, &eq_tuner.mode);
        }

        visual_processor.process(&audio_processor, &eq_tuner.mode);
        eq_tuner.display_ledmatrix(&visual_processor.color_vec);
    }
}