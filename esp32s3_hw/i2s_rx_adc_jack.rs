use esp_idf_hal::{
    gpio::*, i2s::{config::{ClockSource, MclkMultiple::*, SlotMode::*, *}, *}
};

use super::I2sEnum;

pub fn boot_get_driver<'a, const B: usize, const D: usize, const W: usize, const I: usize> (esp32: &mut super::Esp32S3c1, sample_rate: u32) -> I2sDriver<'a, I2sRx> {
        let channel_cfg = Config::new();
        let clk_config = StdClkConfig::new(sample_rate, ClockSource::Pll160M, M512);
        let slot_config: StdSlotConfig = StdSlotConfig::philips_slot_default(DataBitWidth::Bits32, Mono);
        let gpio_cfg = StdGpioConfig::new(false,false,false);
        let i2s_std_config = StdConfig::new(channel_cfg, clk_config, slot_config, gpio_cfg);

        let bclk = esp32.gpio_manager.get_gpio(B);
        let din = esp32.gpio_manager.get_gpio(D);
        let mclk = AnyIOPin::none(); // don't use
        let ws = esp32.gpio_manager.get_gpio(W);
        
        let i2s_choice = esp32.i2s_manager.get_i2s(I);

        let mut i2s_driver = match i2s_choice {
            I2sEnum::I2S0(i2s_peripheral) => I2sDriver::new_std_rx(i2s_peripheral, &i2s_std_config, bclk, din, mclk, ws).unwrap(),
            I2sEnum::I2S1(i2s_peripheral) => I2sDriver::new_std_rx(i2s_peripheral, &i2s_std_config, bclk, din, mclk, ws).unwrap()
        };

        i2s_driver.rx_enable().ok();
        i2s_driver
}