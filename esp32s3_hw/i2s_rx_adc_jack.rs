use esp_idf_hal::{
    gpio::*, i2s::{config::{ClockSource, MclkMultiple::*, SlotMode::*, *}, *}
};

use super::I2sEnum;

pub fn boot_get_driver<'i>( 
    sample_rate: u32,
    i2s_choice: I2sEnum, 
    mut audio_gpios: Vec<AnyIOPin>
    )
    -> I2sDriver<'i, I2sRx> {
        let channel_cfg = Config::new();
        let clk_config = StdClkConfig::new(sample_rate, ClockSource::Pll160M, M512);
        let slot_config: StdSlotConfig = StdSlotConfig::philips_slot_default(DataBitWidth::Bits32, Mono);
        let gpio_cfg = StdGpioConfig::new(false,false,false);
        let i2s_std_config = StdConfig::new(channel_cfg, clk_config, slot_config, gpio_cfg);

        let bclk = audio_gpios.remove(0);
        let din = audio_gpios.remove(0);
        let mclk = AnyIOPin::none();
        let ws = audio_gpios.remove(0);

        let mut i2s_driver = match i2s_choice {
            I2sEnum::I2S0(i2s_peripheral) => I2sDriver::new_std_rx(i2s_peripheral, &i2s_std_config, bclk, din, mclk, ws).unwrap(),
            I2sEnum::I2S1(i2s_peripheral) => I2sDriver::new_std_rx(i2s_peripheral, &i2s_std_config, bclk, din, mclk, ws).unwrap()
        };

        i2s_driver.rx_enable().ok();
        i2s_driver
}