use std::collections::HashMap;

use esp_idf_hal::{
    gpio::*, i2s::*, prelude::Peripherals, modem::Modem
};
use esp_idf_svc::wifi::EspWifi;
use ws2812_esp32_rmt_driver::driver::Ws2812Esp32RmtDriver;

mod i2s_rx_mems_mic;
mod i2s_rx_adc_jack;
mod esp32s3_wifi;
pub mod config;

// macro crates used for gpio definition and retrieval
use seq_macro::seq;
use paste::paste;

/*
Module serves as an overview of available (and proven to work) drivers+configurations for my situation as well as instantiation of drivers.

Struct Esp32S3c1 is a wrapper for the motherboard and uniform access to periphs based on number. It also shows which esp32 input/output facilities are 
currently in play in this project (gpios, i2s, modem, boot button) and keeps track of available gpios and i2s entries.

Mems mic and ac2 jack currently have different configurations (currently only the mclk multiple which is M512 for jack) and have been fiddled with extensively in the past. Separate boot files for those make future fiddling with
config settings per audio device possible. According to its docs the MEMS mic has a 24 bit datawidth but that doesn't work.

Would be nicer to have a ledmatrix driver that works with esp_idf_hal but low prio as current setup works.

Currently not supporting driver shutdown / returning resources as I don't need it.
*/

pub enum I2sEnum {
    I2S0(I2S0),
    I2S1(I2S1)
}

seq!(N in 0..=21 {
    pub struct GpioManager{
    #(
            gpio~N: Option<Gpio~N>,
    )*
    }
});
impl GpioManager {
    // consume the gpio from hashmap and return it. 
    pub fn get_gpio(&mut self, num: u8) -> AnyIOPin {
        self.gpio_hashmap.remove(&num).unwrap()
    }

    pub fn get_gpios(&mut self, nums: Vec<u8>) -> Vec<AnyIOPin> {
        let mut gpios = vec![];
        for num in nums {
            let gpio = self.gpio_hashmap.remove(&num).unwrap();
            gpios.push(gpio)
        }
        gpios
    }
}

pub struct I2sManager {
    pub i2s_hashmap: HashMap<u8, I2sEnum>,
}

impl I2sManager {
    // consume the gpio from hashmap and return it. 
    pub fn get_i2s(&mut self, num:u8) -> I2sEnum {
        self.i2s_hashmap.remove(&num).unwrap()
    }
}

pub struct Esp32S3c1{
    pub gpio_manager: GpioManager,
    pub i2s_manager: I2sManager,
    pub modem: Modem,
}

impl Esp32S3c1 {
    pub fn new() -> Self {
        let periphs = Peripherals::take().unwrap();
        let gpio_manager = seq!(N in 0..=21 {
            GpioManager{
            #(
                    gpio~N: Some(periphs.pins.gpio~N),
            )*
            }
        });
        let mut i2s_hashmap = HashMap::new();
        /*let mut gpio_hashmap = HashMap::new();
        let mut i2s_hashmap = HashMap::new();

        // 22 up to and including 25 are not available on ESP32S3-c1.
        seq!(N in 0..22 {
            #(
                    /* Downgrade turns every specific gpioNUMBER struct into an AnyIOPin.
                    Loses info about the pin (for instance if it's input only or not)
                    so careful when choosing pins.
                    This may need a refactor when projects start using something else than
                    input pins so the idf-hal can protect against choices that won't work. Or
                    just need to pay attention choosing pins. */

                    gpio_hashmap.insert(N, periphs.pins.gpio~N.downgrade());
            )*
        });
        seq!(N in 26..=48 {
            #(
                    gpio_hashmap.insert(N, periphs.pins.gpio~N.downgrade());
            )*
        });
        */
        i2s_hashmap.insert(0, I2sEnum::I2S0(periphs.i2s0));
        i2s_hashmap.insert(1, I2sEnum::I2S1(periphs.i2s1));

        //let gpio_manager = GpioManager{gpio_hashmap};
        let i2s_manager = I2sManager{i2s_hashmap};

        Esp32S3c1 {
            gpio_manager,
            i2s_manager,
            modem: periphs.modem
        }
    }
}

// on board boot button gpio is 0 on my device
pub fn get_on_board_boot_button<'a>(esp32: &mut Esp32S3c1, optional_gpio_num: Option<u8>) -> PinDriver<'a, AnyIOPin, Input> {
    let gpio_num = optional_gpio_num.unwrap_or(0);
    let gpio = esp32.gpio_manager.get_gpio(gpio_num);
    let mut boot_button_driver = PinDriver::input(gpio).unwrap();
    boot_button_driver.set_pull(esp_idf_hal::gpio::Pull::Up).ok(); // on board boot button has a default pull up state
    boot_button_driver
}

// default on my current dev board (ESP32S3-C1) is 48
pub fn get_on_board_led_ws2812_driver(esp32: &mut Esp32S3c1, channel_num: u8, optional_gpio_num: Option<u8>) -> Ws2812Esp32RmtDriver {
    let gpio_din_number = optional_gpio_num.unwrap_or(48);
    // make the gpio unavailable to be picked
    esp32.gpio_manager.gpio_hashmap.remove(&gpio_din_number);
    Ws2812Esp32RmtDriver::new(channel_num, gpio_din_number.into()).unwrap()
}

pub fn get_ws2812ledstrip_driver (esp32: &mut Esp32S3c1, channel_num: u8, gpio_din_number: u8 ) -> Ws2812Esp32RmtDriver {
    // make the gpio unavailable to be picked
    esp32.gpio_manager.gpio_hashmap.remove(&gpio_din_number);
    Ws2812Esp32RmtDriver::new(channel_num, gpio_din_number.into()).unwrap()
}

pub fn get_on_board_wifi_driver<'a>(esp32: &'a mut Esp32S3c1, ssid: &str, password: &str) -> EspWifi<'a> {
    let modem = &mut esp32.modem;
    esp32s3_wifi::boot_get_driver(modem, &ssid, &password)
}

pub fn get_mems_microphone_i2s_driver<'a>( 
    esp32: &mut Esp32S3c1,
    sample_rate: u32,
    i2s_num: u8, 
    bclk_num: u8,
    din_num: u8,
    ws_num: u8
    ) -> I2sDriver<'a, I2sRx> {
        i2s_rx_mems_mic::boot_get_driver(esp32, sample_rate, i2s_num, bclk_num, din_num, ws_num)
}

pub fn get_linejack_i2s_driver<'a, const B: usize, const D: usize, const W: usize, const I: usize> (esp32: &mut Esp32S3c1, sample_rate: u32) -> I2sDriver<'a, I2sRx> {
        i2s_rx_adc_jack::boot_get_driver::<B, D, W, I>(esp32, sample_rate)
}