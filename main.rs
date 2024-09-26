use std::sync::atomic::{AtomicBool, Ordering}; // for the interrupt handling
use std::time::{Duration, SystemTime};

use esp_idf_hal::{delay::FreeRtos, gpio::{PinDriver, AnyIOPin, Input}, i2s::{I2sDriver, I2sRx}};
use ws2812_esp32_rmt_driver::driver::Ws2812Esp32RmtDriver;

mod esp32s3_hw; // driver wrappers for confirmed working on-board and connected hardware in my setup
use esp32s3_hw::Esp32S3c1;

mod audiovisual; // process audio feed and output to led matrix
use audiovisual::graphics::*;

pub enum EqTunerMode {
    Equalizer,
    Tuner
}

// Tracks the interrupt on the boot button
static BOOTTON_PRESSED: AtomicBool = AtomicBool::new(false);

// Passed around a lot and don't change as far as the runtime is concerned
pub const LEDS_MAX_X: usize = 8;
pub const LEDS_MAX_Y: usize = 32;

fn boot_button_callback() {
    BOOTTON_PRESSED.store(true, Ordering::Relaxed);
}

/*
EqTuner keeps track of if we're in equalizer or tuner mode, contains the drivers that are used and is a centralized spot for 
initializing some informational variables relating to the specific project / hardware used.

Current logic is only made for a situation where the number of frequency bins in eq mode is equal to the number of rows in the ledmatrix (I'm 
using vertically placed 8x32 matrix, so 32 freq bins). To change that some minor changes in the frequency visualizer are necessary.
*/
struct EqTuner<'a> {
    mode: EqTunerMode,
    audiobuffer: [u8; 3072],
    audio_driver: I2sDriver<'a, I2sRx>,
    ledmatrix_driver: Ws2812Esp32RmtDriver,
    mode_button_driver: PinDriver<'a, AnyIOPin, Input>,
    
    audio_processor: audiovisual::AudioProcessor,
    visual_processor: audiovisual::VisualProcessor,

    frame_duration: Duration, // ledmatrix starts glitching if this is too short 
    last_visual_update: SystemTime,

    switch_element_pos: i32 // for the switch screen animation
}

impl <'a>EqTuner<'a> {
    fn new(sample_rate:u32, num_frequency_bins: u8) -> Self {
        let mut esp32 = Esp32S3c1::new();
        let audiobuffer = [0; 3072]; // buffer for the sound driver

        let audio_processor = audiovisual::AudioProcessor::new(audiobuffer.len(), num_frequency_bins, sample_rate);
        let visual_processor = audiovisual::VisualProcessor::new(LEDS_MAX_X, LEDS_MAX_Y);

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
            audiobuffer,
            audio_driver,
            ledmatrix_driver,
            mode_button_driver,
            audio_processor,
            visual_processor,
            frame_duration: Duration::from_micros(100000), // 10 fps is more than enough. Won't be exact due to execution times but should be a fast enough constant refresh rate that doesnt glitch the matrix
            last_visual_update: SystemTime::now(),
            switch_element_pos: -16 // start outside
        }
    }

    fn process_audio_buffer(&mut self) {
        let bytes_read = self.audio_driver.read(&mut self.audiobuffer, 1000).unwrap();
        for chunks in self.audiobuffer.chunks(4).take(bytes_read / 4) { 
            // on Esp32S3 for my two devices the MEMS microphone outputted the middle two bytes and garbage in the 1st and 4th. The linejack hardware outputs all 4 useful bytes. Currently working with linejack
            let unprocessed_audio_value = i32::from_le_bytes( [chunks[0], chunks[1], chunks[2], chunks[3]]);
            self.audio_processor.process(unprocessed_audio_value, &self.mode);
        }
    }

    fn process_visuals(&mut self) {
        self.visual_processor.process(&self.audio_processor, &self.mode);
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

    fn display_ledmatrix(&mut self) {
        let now = SystemTime::now();
        let elapsed = now.duration_since(self.last_visual_update).unwrap_or(Duration::ZERO);

        if elapsed >= self.frame_duration {
            self.ledmatrix_driver.write(&self.visual_processor.color_vec).ok();
            self.last_visual_update = now;
        }
    }

    fn display_switch_screen(&mut self) {
        let mut mode_init_screen = match self.mode {
            EqTunerMode::Equalizer => {
                let mut empty_canvas = Vec::with_capacity(LEDS_MAX_X*LEDS_MAX_Y);
                for _ in 0..(LEDS_MAX_X*LEDS_MAX_Y) {
                    empty_canvas.append(&mut vec![1,30,1]);
                }
                empty_canvas
            },
            EqTunerMode::Tuner => {
                let mut empty_canvas = Vec::with_capacity(LEDS_MAX_X*LEDS_MAX_Y);
                for _ in 0..(LEDS_MAX_X*LEDS_MAX_Y) {
                    empty_canvas.append(&mut vec![1,1,5]);
                }
                empty_canvas
            }
        };

        let mut mode_init_animation = mode_init_screen.clone();
        let one_up_graph = vecvec_one_up();
        let eq_graph = convert_vecvecbool_to_xy_rgb_vec(vecvecbool_eq(), RGB{r:0, g:70, b:50});
        let tun_graph = convert_vecvecbool_to_xy_rgb_vec(vecvecbool_tuner(), RGB{r:0, g:70, b:50});

        let mut animation_bg = mode_init_animation.clone();

        match self.mode {
            EqTunerMode::Equalizer => {
                for _ in 0..24 {
                    self.switch_element_pos -= 1;
                    paint_element(&mut animation_bg, &one_up_graph, self.switch_element_pos, 2);
                    paint_element(&mut animation_bg, &eq_graph, 1, 23);

                    self.visual_processor.color_vec = animation_bg.clone(); // replace with an initial screen after switch
                    self.display_ledmatrix();
                    animation_bg = mode_init_animation.clone();
                    FreeRtos::delay_ms(100) // bask in the glory of the switch screen
                }
            },
            EqTunerMode::Tuner => {
                for _ in 0..24 {
                    self.switch_element_pos += 1;
                    paint_element(&mut mode_init_animation, &one_up_graph, self.switch_element_pos, 2);
                    paint_element(&mut mode_init_animation, &tun_graph, 1, 23);
                    
                    self.visual_processor.color_vec = mode_init_animation.clone(); // replace with an initial screen after switch
                    self.display_ledmatrix();
                    mode_init_animation = mode_init_screen.clone();
                    FreeRtos::delay_ms(100) // bask in the glory of the switch screen
                }
            }
        }
        let line_graph = line(LEDS_MAX_X, RGB{r:255, g:216, b:0});
        let dot_graph = dot(RGB{r:40, g: 0, b: 0});
        paint_element(&mut mode_init_screen, &line_graph, 0, 16);
        paint_element(&mut mode_init_screen, &dot_graph, 2, 20);
        paint_element(&mut mode_init_screen, &dot_graph, 4, 20);
        paint_element(&mut mode_init_screen, &dot_graph, 6, 20);

        self.visual_processor.color_vec = mode_init_screen.clone(); // replace with an initial screen after switch
        FreeRtos::delay_ms(200) // bask in the glory of the switch screen
    }
}

fn main() {
    esp_idf_hal::sys::link_patches();
    let mut eq_tuner = EqTuner::new(48000, 32);

    /*
    Main loop: read the audiobuffer from the i2s driver and run the audio processor on it. 

    The visual processor reads audioprocessor output, processes, and outputs the color array (size is ledmatrix_x*ledmatrix_y*3 for g,r,b on every led) 
    */
    loop { 
        FreeRtos::delay_ms(1); // give OS a chance to do some threading
        
        // Switch from equalizer to tuner and back on button presses
        eq_tuner.check_mode_switch();

        eq_tuner.process_audio_buffer();
        eq_tuner.process_visuals();

        eq_tuner.display_ledmatrix();
    }
}