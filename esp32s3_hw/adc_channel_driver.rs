use std::borrow::Borrow;

use esp_idf_sys::*;
use esp_idf_hal::{adc::oneshot::*, adc::oneshot::config::*, adc::attenuation::*, gpio::ADCPin};

use super::{Esp32S3c1, ADCEnum};

use seq_macro::seq;

pub trait AdcChannelWrap{
    fn read(&mut self) -> Result<u16, EspError>;
}

impl <'a, G, A>AdcChannelWrap for AdcChannelDriver<'a, G, A>
where
G: ADCPin,
A: Borrow<AdcDriver<'a, G::Adc>>
{
    fn read(&mut self) -> Result<u16, EspError> {
        self.read()
    }
}

pub fn boot_get_driver<'d>(esp32: &mut Esp32S3c1, gpio_num: u8) -> Box<dyn AdcChannelWrap>
{   
    let config = AdcChannelConfig {
        attenuation: DB_11,
        calibration: true,
        ..Default::default()
    };

    if (1..=10).contains(&gpio_num) {
        let adc_choice = esp32.adc_manager.get_adc_enum(1);
        match adc_choice {
            ADCEnum::ADC1(adc) => {
                let adc_driver = AdcDriver::new(adc).unwrap();
                seq!(N in 1..=10 {
                    match gpio_num {
                        #(
                        N => {
                            let gpio = esp32.gpio_manager.gpio~N.take().unwrap();
                            Box::new(AdcChannelDriver::new(adc_driver, gpio, &config).unwrap())
                        }
                        )* 
                        _ => panic!("Invalid GPIO num")
                    }
                })
            },
            _ => {
                panic!("ADC not available for chosen pin")
            }
        }
    }
    else if (11..=20).contains(&gpio_num) {
        let adc_choice = esp32.adc_manager.get_adc_enum(2);
        match adc_choice {
            ADCEnum::ADC2(adc) => {
                let adc_driver = AdcDriver::new(adc).unwrap();
                seq!(N in 11..=20 {
                    match gpio_num {
                        #(
                        N => {
                            let gpio = esp32.gpio_manager.gpio~N.take().unwrap();
                            Box::new(AdcChannelDriver::new(adc_driver, gpio, &config).unwrap())
                        }
                        )* 
                        _ => panic!("Invalid GPIO num")
                    }
                })
            },
            _ => {
                panic!("ADC not available for chosen pin")
            }
        }
    }
    else {
        panic!("Not a pin with ADC capabilities")
    }
}