use esp_hal::{
    Blocking,
    gpio::interconnect::PeripheralOutput,
    i2c::master::{Config, I2c, Instance},
};
use vector_apps::apps::Controls;

pub struct Nunchuck<'a> {
    i2c: I2c<'a, Blocking>,
}

const NUNCHUCK_ADDR: u8 = 0x52;

fn clamp_joystick(val: i16) -> i8 {
    match val {
        v if v < 0 => -1,
        v if v == 0 => 0,
        v if v > 0 => 1,
        _ => unreachable!(),
    }
}

impl<'a> Nunchuck<'a> {
    pub fn new(
        i2c: impl Instance + 'a,
        sda: impl PeripheralOutput<'a>,
        scl: impl PeripheralOutput<'a>,
    ) -> Self {
        let mut i2c = I2c::new(i2c, Config::default())
            .unwrap()
            .with_sda(sda)
            .with_scl(scl);

        // Send handshake
        i2c.write(NUNCHUCK_ADDR, &[0xF0, 0x55, 0xFB, 0x00]).unwrap();

        Self { i2c }
    }

    pub fn get_input(&mut self) -> Controls {
        self.i2c.write(NUNCHUCK_ADDR, &[0x00]).unwrap();
        let mut result = [0x00; 6];
        self.i2c.read(NUNCHUCK_ADDR, &mut result).unwrap();

        let x = result[0x00] as i16 - 128;
        let y = result[0x01] as i16 - 128;
        let c = result[0x05] & 0b0010 == 0;
        let z = result[0x05] & 0b0001 == 0;

        Controls {
            a: c,
            b: z,
            x: clamp_joystick(x),
            y: clamp_joystick(y),
        }
    }
}
