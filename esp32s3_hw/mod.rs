use std::collections::HashMap;

use esp_idf_hal::{gpio::*, i2s::*, adc::*, modem::Modem, prelude::Peripherals};
use esp_idf_svc::wifi::EspWifi;
use ws2812_esp32_rmt_driver::driver::Ws2812Esp32RmtDriver;

mod i2s_rx_mems_mic;
mod i2s_rx_adc_jack;
mod esp32s3_wifi;
mod adc_channel_driver;
pub use adc_channel_driver::AdcChannelWrap;
pub mod config;
// macro crate used for gpio retrieval
use seq_macro::seq;

/*
Module serves as an overview of available (and proven to work) drivers+configurations for my situation as well as instantiation of drivers.

Struct Esp32S3c1 is a wrapper for the motherboard and uniform access to periphs based on number. It also shows which esp32 input/output facilities are 
currently in play in my projects (gpios, i2s, adc, modem, boot button) and keeps track of available gpios, i2s and adc entries.

Mems mic and ac2 jack currently have different configurations (currently only the mclk multiple which is M512 for jack) and have been fiddled with extensively in the past. Separate boot files for those make future fiddling with
config settings per audio device possible. According to its docs the MEMS mic has a 24 bit datawidth but that doesn't work.

Would be nicer to have a ledmatrix driver that works with esp_idf_hal but low prio as current setup works.

Currently not supporting driver shutdown / returning resources as I don't need it.
*/

pub struct Esp32S3c1{
    gpio_manager: GpioManager,
    i2s_manager: I2sManager,
    adc_manager: ADCManager,
    modem: Modem,
}

impl Esp32S3c1 {
    pub fn new(periphs: Peripherals) -> Self {
        let mut i2s_hashmap = HashMap::new();    
        i2s_hashmap.insert(0, I2sEnum::I2S0(periphs.i2s0));
        i2s_hashmap.insert(1, I2sEnum::I2S1(periphs.i2s1));
        
        let mut adc_hashmap = HashMap::new();    
        adc_hashmap.insert(1, ADCEnum::ADC1(periphs.adc1));
        adc_hashmap.insert(2, ADCEnum::ADC2(periphs.adc2));

        let modem = periphs.modem;
        let i2s_manager = I2sManager{i2s_hashmap};
        let adc_manager = ADCManager{adc_hashmap};
        let gpio_manager= GpioManager::new(periphs.pins);

        Esp32S3c1 {
            gpio_manager,
            i2s_manager,
            adc_manager,
            modem
        }
    }
}

struct I2sManager {
    i2s_hashmap: HashMap<u8, I2sEnum>,
}

impl I2sManager {
    // consume the i2s from hashmap and return it. 
    fn get_i2s_enum(&mut self, num:u8) -> I2sEnum {
        self.i2s_hashmap.remove(&num).unwrap()
    }
}

pub enum I2sEnum {
    I2S0(I2S0),
    I2S1(I2S1)
}

struct ADCManager {
    adc_hashmap: HashMap<u8, ADCEnum>
}

impl ADCManager {
    // consume the adc from hashmap and return it.
    fn get_adc_enum(&mut self, num: u8) -> ADCEnum {
        self.adc_hashmap.remove(&num).unwrap()
    }
}

pub enum ADCEnum {
    ADC1(ADC1),
    ADC2(ADC2)
}

struct GpioManager {
    gpio0: Option<Gpio0>,
    gpio1: Option<Gpio1>,
    gpio2: Option<Gpio2>,
    gpio3: Option<Gpio3>,
    gpio4: Option<Gpio4>,
    gpio5: Option<Gpio5>,
    gpio6: Option<Gpio6>,
    gpio7: Option<Gpio7>,
    gpio8: Option<Gpio8>,
    gpio9: Option<Gpio9>,
    gpio10: Option<Gpio10>,
    gpio11: Option<Gpio11>,
    gpio12: Option<Gpio12>,
    gpio13: Option<Gpio13>,
    gpio14: Option<Gpio14>,
    gpio15: Option<Gpio15>,
    gpio16: Option<Gpio16>,
    gpio17: Option<Gpio17>,
    gpio18: Option<Gpio18>,
    gpio19: Option<Gpio19>,
    gpio20: Option<Gpio20>,
    gpio21: Option<Gpio21>,
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

seq!(N in 0..=48 {
    impl GpioManager {
        fn new(pins: Pins) -> Self {
            GpioManager {
                gpio0: Some(pins.gpio0),
                gpio1: Some(pins.gpio1),
                gpio2: Some(pins.gpio2),
                gpio3: Some(pins.gpio3),
                gpio4: Some(pins.gpio4),
                gpio5: Some(pins.gpio5),
                gpio6: Some(pins.gpio6),
                gpio7: Some(pins.gpio7),
                gpio8: Some(pins.gpio8),
                gpio9: Some(pins.gpio9),
                gpio10: Some(pins.gpio10),
                gpio11: Some(pins.gpio11),
                gpio12: Some(pins.gpio12),
                gpio13: Some(pins.gpio13),
                gpio14: Some(pins.gpio14),
                gpio15: Some(pins.gpio15),
                gpio16: Some(pins.gpio16),
                gpio17: Some(pins.gpio17),
                gpio18: Some(pins.gpio18),
                gpio19: Some(pins.gpio19),
                gpio20: Some(pins.gpio20),
                gpio21: Some(pins.gpio21),
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
            if (22..=25).contains(&num) {
                panic!("Gpio 22-25 are not available")
            }
            match num {
                #(
                    N => self.gpio~N.take().unwrap().downgrade_input(),
                )*
                _ => panic!("Gpio not found")
            }
        }

        fn get_gpio_output(&mut self, num: u8) -> AnyOutputPin {
            if (22..=25).contains(&num) {
                panic!("Gpio 22-25 are not available")
            }
            match num {
                #(
                    N => self.gpio~N.take().unwrap().downgrade_output(),
                )*
                _ => panic!("Gpio not found")
            }
        }

        fn get_gpio_input_output(&mut self, num: u8) -> AnyIOPin {
            if (22..=25).contains(&num) {
                panic!("Gpio 22-25 are not available")
            }
            match num {
                #(
                    N => self.gpio~N.take().unwrap().downgrade(),
                )*
                _ => panic!("Gpio not found")
            }
        }
    }
});

// on board boot button gpio is 0 on my device
pub fn get_on_board_boot_button<'a>(esp32: &mut Esp32S3c1, optional_gpio_num: Option<u8>) -> PinDriver<'a, AnyIOPin, Input> {
    let gpio_num = optional_gpio_num.unwrap_or(0);
    let gpio = esp32.gpio_manager.get_gpio_input_output(gpio_num);
    let mut boot_button_driver = PinDriver::input(gpio).unwrap();
    boot_button_driver.set_pull(esp_idf_hal::gpio::Pull::Up).ok(); // on board boot button has a default pull up state
    boot_button_driver
}

pub fn get_pin_driver_input_button<'a>(esp32: &mut Esp32S3c1, gpio_num: u8) -> PinDriver<'a, AnyIOPin, Input> {
    let gpio = esp32.gpio_manager.get_gpio_input_output(gpio_num);
    let mut pin_driver = PinDriver::input(gpio).expect("Failed to get pin driver");
    pin_driver.set_pull(esp_idf_hal::gpio::Pull::Up).expect("Failed to set pin driver pull");
    pin_driver
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

// the called function automatically selects the right ADC channel for the selected GPIO pin (for ESP32S3c1)
pub fn get_adc_channel_driver<'a>(
    esp32: &mut Esp32S3c1,
    gpio_num: u8
)  -> Box<dyn AdcChannelWrap> {
    adc_channel_driver::boot_get_driver(esp32, gpio_num)
}