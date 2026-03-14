use esp_hal::analog::dac::Dac;
use esp_hal::gpio::interconnect::PeripheralOutput;
use esp_hal::gpio::{
    DriveMode, Input, InputConfig, InputPin, Level, Output, OutputConfig, OutputPin,
};
use esp_hal::ledc::channel::{Channel, ChannelHW, ChannelIFace};
use esp_hal::ledc::timer::Timer;
use esp_hal::ledc::{LSGlobalClkSource, Ledc, LowSpeed, channel};
use esp_hal::peripherals::{DAC1, DAC2, GPIO17, GPIO18};

use vector_apps::point::Point;

pub struct Lasers<'a> {
    enable: Output<'a>,
    interlock: Input<'a>,
    red: Channel<'a, LowSpeed>,
    green: Channel<'a, LowSpeed>,
    blue: Channel<'a, LowSpeed>,
    x: Dac<'a, DAC1<'a>>,
    y: Dac<'a, DAC2<'a>>,
}

impl<'a> Lasers<'a> {
    pub fn new(
        enable: impl OutputPin + 'a,
        interlock: impl InputPin + 'a,
        red: impl PeripheralOutput<'a>,
        green: impl PeripheralOutput<'a>,
        blue: impl PeripheralOutput<'a>,
        mut ledc: Ledc<'a>,
        timer: &'a Timer<'a, LowSpeed>,
        dac_x: DAC1<'a>,
        pin_x: GPIO17<'a>,
        dac_y: DAC2<'a>,
        pin_y: GPIO18<'a>,
    ) -> Self {
        ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

        let en_output = Output::new(enable, Level::Low, OutputConfig::default());
        let interlock_input = Input::new(interlock, InputConfig::default());

        let mut red_ch = ledc.channel(channel::Number::Channel0, red);
        red_ch
            .configure(channel::config::Config {
                timer,
                duty_pct: 10,
                drive_mode: DriveMode::PushPull,
            })
            .unwrap();

        let mut green_ch = ledc.channel(channel::Number::Channel1, green);
        green_ch
            .configure(channel::config::Config {
                timer,
                duty_pct: 10,
                drive_mode: DriveMode::PushPull,
            })
            .unwrap();

        let mut blue_ch = ledc.channel(channel::Number::Channel2, blue);
        blue_ch
            .configure(channel::config::Config {
                timer,
                duty_pct: 10,
                drive_mode: DriveMode::PushPull,
            })
            .unwrap();

        Self {
            enable: en_output,
            interlock: interlock_input,
            red: red_ch,
            green: green_ch,
            blue: blue_ch,
            x: Dac::new(dac_x, pin_x),
            y: Dac::new(dac_y, pin_y),
        }
    }

    pub fn is_interlock_active(&self) -> bool {
        self.interlock.is_high()
    }

    pub fn display(&mut self, p: &Point) {
        self.enable.set_high();

        self.x.write(255 - p.x);
        self.y.write(255 - p.y);
        self.red.set_duty_hw(p.color.0 as u32);
        self.green.set_duty_hw(p.color.1 as u32);
        self.blue.set_duty_hw(p.color.2 as u32);
    }
}

impl<'a> Drop for Lasers<'a> {
    fn drop(&mut self) {
        self.enable.set_low();
    }
}
