use esp_hal::analog::dac::Dac;
use esp_hal::gpio::{Level, Output, OutputConfig, OutputPin};
use esp_hal::peripherals::{DAC1, DAC2, GPIO17, GPIO18};

use crate::point::{COLOR_BLUE, COLOR_GREEN, COLOR_RED, Point};

pub struct Lasers<'a> {
    red: Output<'a>,
    green: Output<'a>,
    blue: Output<'a>,
    x: Dac<'a, DAC2<'a>>,
    y: Dac<'a, DAC1<'a>>,
}

impl<'a> Lasers<'a> {
    pub fn new(
        red: impl OutputPin + 'a,
        green: impl OutputPin + 'a,
        blue: impl OutputPin + 'a,
        dac_x: DAC2<'a>,
        pin_x: GPIO18<'a>,
        dac_y: DAC1<'a>,
        pin_y: GPIO17<'a>,
    ) -> Self {
        Self {
            red: Output::new(red, Level::Low, OutputConfig::default()),
            green: Output::new(green, Level::Low, OutputConfig::default()),
            blue: Output::new(blue, Level::Low, OutputConfig::default()),
            x: Dac::new(dac_x, pin_x),
            y: Dac::new(dac_y, pin_y),
        }
    }

    pub fn display(&mut self, p: &Point) {
        self.x.write(255 - p.x);
        self.y.write(255 - p.y);

        if (p.color & COLOR_RED) != 0 {
            self.red.set_high();
        } else {
            self.red.set_low();
        }

        if (p.color & COLOR_GREEN) != 0 {
            self.green.set_high();
        } else {
            self.green.set_low();
        }

        if (p.color & COLOR_BLUE) != 0 {
            self.blue.set_high();
        } else {
            self.blue.set_low();
        }
    }
}
