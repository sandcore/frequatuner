use std::sync::atomic::{AtomicBool, Ordering}; // for the interrupt handling
use std::time::{Duration, SystemTime};

use esp_idf_hal::{delay::FreeRtos, gpio::{PinDriver, AnyIOPin, Input}, i2s::{I2sDriver, I2sRx}};
use ws2812_esp32_rmt_driver::driver::Ws2812Esp32RmtDriver;

mod esp32s3_hw; // driver wrappers for confirmed working on-board and connected hardware in my setup
use esp32s3_hw::Esp32S3c1;

mod audiovisual; // process audio feed and output to led matrix
use audiovisual::graphics::{self, *};

pub enum EqTunerModeEnum {
    Equalizer,
    Tuner
}

// Used by the interrupt on the boot button
static BOOTTON_PRESSED: AtomicBool = AtomicBool::new(false);

/* Passed around a lot and don't change as far as the runtime is concerned.
Current logic for equalizer mode is only made for a situation where the number of frequency bins in eq mode is equal to the number of rows in the ledmatrix (I'm 
using vertically placed so 32 freq bins). To change that some minor changes in the frequency visualizer are necessary.*/
pub const LEDS_MAX_X: usize = 8;
pub const LEDS_MAX_Y: usize = 32;

fn boot_button_callback() {
    BOOTTON_PRESSED.store(true, Ordering::Relaxed);
}

// Manages setup of, and direct interactions with, hardware drivers
struct HwCommander<'a> {
    audiobuffer: [u8; 3072],
    audio_driver: I2sDriver<'a, I2sRx>,
    ledmatrix_driver: Ws2812Esp32RmtDriver,
    mode_button_driver: PinDriver<'a, AnyIOPin, Input>,
    frame_duration: Duration, // ledmatrix starts glitching if this is too short 
    last_visual_update: SystemTime,
}
impl <'a>HwCommander<'a> {
    fn new(sample_rate:u32) -> HwCommander<'a> {
        let mut esp32 = Esp32S3c1::new();
        let audiobuffer = [0u8; 3072]; // buffer for the sound driver

        let audio_driver = esp32s3_hw::get_linejack_i2s_driver(&mut esp32, sample_rate, 0, 5, 7, 6);        
        let ledmatrix_driver = esp32s3_hw::get_ws2812ledstrip_driver(&mut esp32, 3, 18);
        
        // set up the mode switch button and set an interrupt on it
        let mut mode_button_driver = esp32s3_hw::get_on_board_boot_button(&mut esp32, None);
        mode_button_driver.set_interrupt_type(esp_idf_hal::gpio::InterruptType::NegEdge).ok();
        unsafe {
            mode_button_driver.subscribe(boot_button_callback).expect("Interrupt subscribe failed");
        }
        mode_button_driver.enable_interrupt().ok();

        HwCommander {
            audiobuffer,
            audio_driver,
            ledmatrix_driver,
            mode_button_driver,
            frame_duration: Duration::from_micros(100000), // 10 fps is more than enough. Won't be exact due to execution times but should be a fast enough constant refresh rate that doesnt glitch the matrix
            last_visual_update: SystemTime::now(),
        }
    }

    fn re_enable_interrupt(&mut self) {
        self.mode_button_driver.enable_interrupt().ok();
    }

    fn read_audio_buffer(&mut self) -> Vec<f32> {
        let bytes_read = self.audio_driver.read(&mut self.audiobuffer, 1000).unwrap();
        let mut audio_values = Vec::with_capacity(self.audiobuffer.len() / 4);

        for chunks in self.audiobuffer.chunks(4).take(bytes_read / 4) { 
            // on Esp32S3 for my two devices the MEMS microphone outputted the middle two bytes and garbage in the 1st and 4th. The linejack hardware outputs all 4 useful bytes. Currently working with linejack
            let unprocessed_audio_value = i32::from_le_bytes( [chunks[0], chunks[1], chunks[2], chunks[3]]);
            let audio_value = unprocessed_audio_value as f64 / (i32::MAX) as f64; // normalized, between 0 and 1
            audio_values.push(audio_value as f32);
        }
        audio_values
    }

    fn display_ledmatrix(&mut self) {
        let now = SystemTime::now();
        let elapsed = now.duration_since(self.last_visual_update).unwrap_or(Duration::ZERO);

        if elapsed >= self.frame_duration {
            self.ledmatrix_driver.write(&self.visual_processor.color_vec).ok();
            self.last_visual_update = now;
        }
    }
}

/*
Keeps track of if we're in equalizer or tuner mode, switches mode
*/
struct FrequalizerMode {
    mode: EqTunerModeEnum
}
impl FrequalizerMode {
    fn new() -> FrequalizerMode {
        FrequalizerMode {
            mode: EqTunerModeEnum::Equalizer
        }
    }
    fn check_switch_mode(&mut self) -> bool {
        if BOOTTON_PRESSED.load(Ordering::Relaxed) {
            BOOTTON_PRESSED.store(false, Ordering::Relaxed);
            self.switch_mode(None);
            true
        }
        else {
            false
        }
    }

    fn switch_mode(&mut self, desired_mode: Option<EqTunerModeEnum>) {
        match desired_mode {
            Some(EqTunerModeEnum::Equalizer) => self.mode = EqTunerModeEnum::Equalizer,
            Some(EqTunerModeEnum::Tuner) => self.mode = EqTunerModeEnum::Tuner,
            None => {
                match self.mode {
                    EqTunerModeEnum::Equalizer => self.mode = EqTunerModeEnum::Tuner,
                    EqTunerModeEnum::Tuner => self.mode = EqTunerModeEnum::Equalizer
                }
            }
        }
        graphics::display_switch_animation(&self.mode);
    }
}

fn main() {
    esp_idf_hal::sys::link_patches();
    let hw_commander = HwCommander::new(48000);
    let fr_mode = FrequalizerMode::new();

    /*
    Main loop: read the audiobuffer and run the audio processor on it. 

    The visual processor reads audioprocessor output, processes, and outputs a color array (size is ledmatrix_x*ledmatrix_y*3 for g,r,b on every led) 
    */
    loop { 
        FreeRtos::delay_ms(1); // give OS a chance to do some threading
        
        // Switch from equalizer to tuner and back on button presses. If the button was pressed the interrupt needs to be re-enabled
        if fr_mode.check_switch_mode() {hw_commander.re_enable_interrupt();}

        eq_tuner.process_audio_buffer();
        eq_tuner.process_visuals();

        eq_tuner.display_ledmatrix();
    }
}