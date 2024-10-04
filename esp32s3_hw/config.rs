// Fixed settings as far as the runtime is concerned

/* Current logic for equalizer mode is only made for a situation where the number of frequency bins in eq mode is equal to the number of rows in the ledmatrix (I'm 
using vertically placed so 32 freq bins). To change that some minor changes in the frequency visualizer are necessary.*/
pub const LEDS_MAX_X: usize = 8;
pub const LEDS_MAX_Y: usize = 32;

// Led matrix gpio + channel
pub const LEDS_IN: u8 = 18;
pub const LEDS_CHANNEL: u8 = 3;

// Audio input gpio + i2s
pub const AUDIO_IN_BCLK: u8 = 5;
pub const AUDIO_IN_DIN: u8 = 7;
pub const AUDIO_IN_WS: u8 = 6;
pub const AUDIO_IN_I2S: u8 = 0;

pub const AUDIO_SAMPLE_RATE: u32 = 48000;

pub const EXTERNAL_MODE_BUTTON_USE: bool = true;
pub const EXTERNAL_MODE_BUTTON_GPIO_NUM: u8 = 3;

pub const GAIN_KNOB_GPIO: u8 = 16;