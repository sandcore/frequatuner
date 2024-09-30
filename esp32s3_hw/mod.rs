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

// macro crate used for gpio definition and retrieval
use seq_macro::seq;

/*
Module serves as an overview of available (and proven to work) drivers+configurations for my situation as well as instantiation of drivers.

Struct Esp32S3c1 is a wrapper for the motherboard and uniform access to periphs based on number. It also shows which esp32 input/output facilities are 
currently in play in this project (gpios, i2s, modem, boot button) and keeps track of available gpios and i2s entries.

Mems mic and ac2 jack currently have different configurations (currently only the mclk multiple which is M512 for jack) and have been fiddled with extensively in the past. Separate boot files for those make future fiddling with
config settings per audio device possible. According to its docs the MEMS mic has a 24 bit datawidth but that doesn't work.

Would be nicer to have a ledmatrix driver that works with esp_idf_hal but low prio as current setup works.

Currently not supporting driver shutdown / returning resources as I don't need it.
*/

pub struct Esp32S3c1{
    gpio_manager: GpioManager,
    i2s_manager: I2sManager,
    modem: Modem,
}

impl Esp32S3c1 {
    pub fn new() -> Self {
        let periphs = Peripherals::take().unwrap();

        let mut i2s_hashmap = HashMap::new();    
        i2s_hashmap.insert(0, I2sEnum::I2S0(periphs.i2s0));
        i2s_hashmap.insert(1, I2sEnum::I2S1(periphs.i2s1));

        let modem = periphs.modem;
        let i2s_manager = I2sManager{i2s_hashmap};
        let gpio_manager= GpioManager::new(periphs.pins);

        Esp32S3c1 {
            gpio_manager,
            i2s_manager,
            modem
        }
    }
}

pub enum I2sEnum {
    I2S0(I2S0),
    I2S1(I2S1)
}

seq!(N in 0..=21 { // want to render 0..=48 while skipping 22-25 but seq doesnt allow it
    struct GpioManager {
        #(
        gpio~N: Option<Gpio~N>,
        )*
    gpio22: Option<AnyIOPin>,
    gpio23: Option<AnyIOPin>,
    gpio24: Option<AnyIOPin>,
    gpio25: Option<AnyIOPin>,
    gpio26: Option<Gpio26>,
    gpio27: Option<Gpio27>,
    gpio28: Option<Gpio28>,
    gpio29: Option<Gpio29>,
    gpio30: Option<Gpio30>,
    gpio31: Option<Gpio31>,
    gpio32: Option<Gpio32>,
    gpio33: Option<Gpio33>,
    gpio34: Option<Gpio34>,
    gpio35: Option<Gpio35>,
    gpio36: Option<Gpio36>,
    gpio37: Option<Gpio37>,
    gpio38: Option<Gpio38>,
    gpio39: Option<Gpio39>,
    gpio40: Option<Gpio40>,
    gpio41: Option<Gpio41>,
    gpio42: Option<Gpio42>,
    gpio43: Option<Gpio43>,
    gpio44: Option<Gpio44>,
    gpio45: Option<Gpio45>,
    gpio46: Option<Gpio46>,
    gpio47: Option<Gpio47>,
    gpio48: Option<Gpio48>,   
    }
});

seq!(N in 0..=21 {
    impl GpioManager {
        fn new(pins: Pins) -> Self {
            GpioManager {
                #(
                gpio~N: Some(pins.gpio~N),
                )*
                gpio22: None,
                gpio23: None,
                gpio24: None,
                gpio25: None,
                gpio26: Some(pins.gpio26),
                gpio27: Some(pins.gpio27),
                gpio28: Some(pins.gpio28),
                gpio29: Some(pins.gpio29),
                gpio30: Some(pins.gpio30),
                gpio31: Some(pins.gpio31),
                gpio32: Some(pins.gpio32),
                gpio33: Some(pins.gpio33),
                gpio34: Some(pins.gpio34),
                gpio35: Some(pins.gpio35),
                gpio36: Some(pins.gpio36),
                gpio37: Some(pins.gpio37),
                gpio38: Some(pins.gpio38),
                gpio39: Some(pins.gpio39),
                gpio40: Some(pins.gpio40),
                gpio41: Some(pins.gpio41),
                gpio42: Some(pins.gpio42),
                gpio43: Some(pins.gpio43),
                gpio44: Some(pins.gpio44),
                gpio45: Some(pins.gpio45),
                gpio46: Some(pins.gpio46),
                gpio47: Some(pins.gpio47),
                gpio48: Some(pins.gpio48),
            }
        }

        fn get_gpio_input(&mut self, num: u8) -> AnyInputPin {
            match num {
                #(
                    N => self.gpio~N.take().unwrap().downgrade_input(),
                )*
                _ => panic!("Gpio not found")
            }
        }
        fn get_gpio_output(&mut self, num: u8) -> AnyOutputPin {
            match num {
                #(
                    N => self.gpio~N.take().unwrap().downgrade_output(),
                )*
                _ => panic!("Gpio not found")
            }
        }
        fn get_gpio_input_output(&mut self, num: u8) -> AnyIOPin {
            match num {
                #(
                    N => self.gpio~N.take().unwrap().downgrade(),
                )*
                _ => panic!("Gpio not found")
            }
        }
    }
});

pub struct I2sManager {
    i2s_hashmap: HashMap<u8, I2sEnum>,
}

impl I2sManager {
    // consume the i2s from hashmap and return it. 
    fn get_i2s_enum(&mut self, num:u8) -> I2sEnum {
        self.i2s_hashmap.remove(&num).unwrap()
    }
}


// on board boot button gpio is 0 on my device
pub fn get_on_board_boot_button<'a>(esp32: &mut Esp32S3c1, optional_gpio_num: Option<u8>) -> PinDriver<'a, AnyIOPin, Input> {
    let gpio_num = optional_gpio_num.unwrap_or(0);
    let gpio = esp32.gpio_manager.get_gpio_input_output(gpio_num);
    let mut boot_button_driver = PinDriver::input(gpio).unwrap();
    boot_button_driver.set_pull(esp_idf_hal::gpio::Pull::Up).ok(); // on board boot button has a default pull up state
    boot_button_driver
}

// default on my current dev board (ESP32S3-C1) is 48
pub fn get_on_board_led_ws2812_driver(esp32: &mut Esp32S3c1, channel_num: u8, optional_gpio_num: Option<u8>) -> Ws2812Esp32RmtDriver {
    let gpio_din_number = optional_gpio_num.unwrap_or(48);
    // make the gpio unavailable to be picked
    esp32.gpio_manager.get_gpio_input_output(gpio_din_number);
    Ws2812Esp32RmtDriver::new(channel_num, gpio_din_number.into()).unwrap()
}

pub fn get_ws2812ledstrip_driver (esp32: &mut Esp32S3c1, channel_num: u8, gpio_din_number: u8 ) -> Ws2812Esp32RmtDriver {
    // make the gpio unavailable to be picked
    esp32.gpio_manager.get_gpio_input_output(gpio_din_number);
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

pub fn get_linejack_i2s_driver<'a>( 
    esp32: &mut Esp32S3c1,
    sample_rate: u32,
    i2s_num: u8, 
    bclk_num: u8,
    din_num: u8,
    ws_num: u8
    ) -> I2sDriver<'a, I2sRx> {
        i2s_rx_adc_jack::boot_get_driver(esp32, sample_rate, i2s_num, bclk_num, din_num, ws_num)
}