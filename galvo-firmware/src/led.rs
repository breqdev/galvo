use esp_hal::{
    gpio::{Level, Output, OutputConfig, OutputPin},
    peripherals::RMT,
    rmt::{PulseCode, Rmt},
    time::Rate,
};
use esp_hal_smartled::SmartLedsAdapter;
use smart_leds::RGB8;
use smart_leds::SmartLedsWrite;
use static_cell::ConstStaticCell;

pub struct IndicatorLed<'a> {
    power: Output<'a>,
    pixel: SmartLedsAdapter<'a, 25>,
}

impl<'a> IndicatorLed<'a> {
    pub fn new(power_pin: impl OutputPin + 'a, rmt: RMT<'a>, led_pin: impl OutputPin + 'a) -> Self {
        let mut power = Output::new(power_pin, Level::High, OutputConfig::default());
        power.set_high();

        static RMT_BUF: ConstStaticCell<[PulseCode; 25]> =
            ConstStaticCell::new([PulseCode::end_marker(); 25]);

        let frequency = Rate::from_mhz(80);
        let rmt = Rmt::new(rmt, frequency).expect("Failed to initialize RMT0");
        let pixel = SmartLedsAdapter::new(rmt.channel0, led_pin, RMT_BUF.take());

        Self { power, pixel }
    }

    pub fn set_color(&mut self, color: RGB8) {
        self.pixel.write([color]).unwrap();
    }
}

impl Drop for IndicatorLed<'_> {
    fn drop(&mut self) {
        self.power.set_low();
    }
}
