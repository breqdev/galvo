use esp_hal::{
    Blocking,
    gpio::interconnect::PeripheralOutput,
    i2c::master::{Config, I2c, Instance},
};
use log::{error, info};
use vector_apps::apps::Controls;

pub struct Joystick {
    x: i8,
    y: i8,
}

pub enum WiiAccessoryReport {
    Nunchuck {
        joy: Joystick,
        /// 10-bit accelerometer data (X, Y, Z)
        accel: (u16, u16, u16),
        /// "C" (upper) button
        c: bool,
        /// "Z" (lower) button
        z: bool,
    },
    ClassicController {
        left: Joystick,
        right: Joystick,
        lt: u8,
        rt: u8,
        dpad: Joystick,

        // Buttons
        zl: bool,
        zr: bool,
        a: bool,
        b: bool,
        x: bool,
        y: bool,
        plus: bool,
        minus: bool,
        home: bool,
    },
    NoConnection {},
}

impl Into<Controls> for WiiAccessoryReport {
    fn into(self) -> Controls {
        match self {
            Self::Nunchuck { joy, c, z, .. } => Controls {
                x: joy.x,
                y: joy.y,
                a: c,
                b: z,
            },
            Self::ClassicController { left, a, b, .. } => Controls {
                x: left.x,
                y: left.y,
                a,
                b,
            },
            Self::NoConnection {} => Controls {
                x: 0,
                y: 0,
                a: false,
                b: false,
            },
        }
    }
}

pub enum WiiAccessory<'a> {
    Nunchuck { i2c: I2c<'a, Blocking> },
    ClassicController { i2c: I2c<'a, Blocking> },
    NoConnection {},
}

const WII_ACCESSORY_ADDR: u8 = 0x52;

impl<'a> WiiAccessory<'a> {
    pub fn new(
        i2c: impl Instance + 'a,
        sda: impl PeripheralOutput<'a>,
        scl: impl PeripheralOutput<'a>,
    ) -> Self {
        let mut i2c = I2c::new(i2c, Config::default())
            .unwrap()
            .with_sda(sda)
            .with_scl(scl);

        // Disable "encryption"
        // Write 0x55 to 0xF0, then 0x00 to 0xFB
        // http://wiibrew.org/wiki/Wiimote/Extension_Controllers#The_New_Way
        match i2c.write(WII_ACCESSORY_ADDR, &[0xF0, 0x55, 0xFB, 0x00]) {
            Ok(()) => {}
            Err(_) => return Self::NoConnection {},
        }

        // Identify the type of controller
        i2c.write(WII_ACCESSORY_ADDR, &[0xFA]).unwrap();
        let mut identifier = [0x00; 6];
        i2c.read(WII_ACCESSORY_ADDR, &mut identifier).unwrap();

        let identifier = u64::from_be_bytes([
            0x00,
            0x00,
            identifier[0],
            identifier[1],
            identifier[2],
            identifier[3],
            identifier[4],
            identifier[5],
        ]);

        match identifier {
            0x0000_A420_0000 => {
                // Nunchuck
                Self::Nunchuck { i2c }
            }
            0x0000_A420_0101 => {
                // Classic Controller
                Self::ClassicController { i2c }
            }
            0x0100_A420_0101 => {
                // Classic Controller Pro
                Self::ClassicController { i2c }
            }
            0xFF00_A420_0013 => {
                // Drawsome Graphics Tablet
                error!("Drawsome Graphics Tablet not supported");
                Self::NoConnection {}
            }
            0x0000_A420_0103 => {
                // Guitar Hero Guitar
                error!("Guitar Hero Guitar not supported");
                Self::NoConnection {}
            }
            0x0100_A420_0103 => {
                // Guitar Hero Drums
                error!("Guitar Hero Drums not supported");
                Self::NoConnection {}
            }
            0x0300_A420_0103 => {
                // DJ Hero Turntable
                error!("DJ Hero Turntable not supported");
                Self::NoConnection {}
            }
            0x0000_A420_0111 => {
                // Taiko Drum Controller
                error!("Taiko Drum Controller not supported");
                Self::NoConnection {}
            }
            0xFF00_A420_0112 => {
                // uDraw GameTablet
                error!("uDraw GameTablet not supported");
                Self::NoConnection {}
            }
            0x0000_A420_0310 => {
                // Shinkansen Controller
                error!("Shinkansen Controller not supported");
                Self::NoConnection {}
            }
            unknown => {
                error!("Unknown controller model: {}", unknown);
                Self::NoConnection {}
            }
        }
    }

    pub fn get_input(&mut self) -> WiiAccessoryReport {
        match self {
            Self::Nunchuck { i2c } => {
                // http://wiibrew.org/wiki/Wiimote/Extension_Controllers/Nunchuck
                i2c.write(WII_ACCESSORY_ADDR, &[0x00]).unwrap();
                let mut result = [0x00; 6];
                i2c.read(WII_ACCESSORY_ADDR, &mut result).unwrap();

                let x = (result[0x00] as i16 - 128) as i8;
                let y = (result[0x01] as i16 - 128) as i8;

                let accel_x = (result[0x02] as u16) << 2 | (result[0x05] as u16 & 0b1100) >> 2;
                let accel_y =
                    (result[0x03] as u16) << 2 | (result[0x05] as u16 & 0b110000) >> 4 as u16;
                let accel_z = (result[0x04] as u16) << 2 | (result[0x05] as u16 & 0b11000000) >> 6;

                let c = result[0x05] & 0b0010 == 0;
                let z = result[0x05] & 0b0001 == 0;

                WiiAccessoryReport::Nunchuck {
                    joy: Joystick { x, y },
                    accel: (accel_x, accel_y, accel_z),
                    c,
                    z,
                }
            }
            Self::ClassicController { i2c } => {
                // http://wiibrew.org/wiki/Wiimote/Extension_Controllers/Nunchuck
                // Assuming data format 0x01

                i2c.write(WII_ACCESSORY_ADDR, &[0x00]).unwrap();
                let mut result = [0x00; 6];
                i2c.read(WII_ACCESSORY_ADDR, &mut result).unwrap();

                let left = {
                    let x = result[0x00] & 0b111111;
                    let y = result[0x01] & 0b111111;

                    let x = (x as i16 - 32) as i8;
                    let y = (y as i16 - 32) as i8;

                    Joystick { x, y }
                };
                let right = {
                    let x = (result[0x00] & 0b1100_0000) >> 3
                        | (result[0x01] & 0b1100_0000) >> 5
                        | (result[0x02] & 0b1000_0000) >> 7;
                    let y = result[0x02] & 0b0001_1111;

                    let x = (x as i16 - 16) as i8;
                    let y = (y as i16 - 16) as i8;

                    Joystick { x, y }
                };

                let dpad = {
                    let l = result[0x05] & 0b0000_0010 == 0;
                    let r = result[0x04] & 0b1000_0000 == 0;
                    let u = result[0x05] & 0b0000_0010 == 0;
                    let d = result[0x04] & 0b0100_0000 == 0;

                    let x = match (l, r) {
                        (false, false) => 0,
                        (false, true) => 1,
                        (true, false) => -1,
                        (true, true) => 0,
                    };

                    let y = match (u, d) {
                        (false, false) => 0,
                        (false, true) => 1,
                        (true, false) => -1,
                        (true, true) => 0,
                    };

                    Joystick { x, y }
                };

                let lt = (result[0x02] & 0b0110_0000) >> 2 | (result[0x03] & 0b1110_0000) >> 5;
                let rt = result[0x03] & 0b0001_1111;

                WiiAccessoryReport::ClassicController {
                    left,
                    right,
                    lt,
                    rt,
                    dpad,
                    zl: result[0x05] & 0b1000_0000 == 0,
                    zr: result[0x05] & 0b0000_0100 == 0,
                    a: result[0x05] & 0b0001_0000 == 0,
                    b: result[0x05] & 0b0100_0000 == 0,
                    x: result[0x05] & 0b0000_1000 == 0,
                    y: result[0x05] & 0b0010_0000 == 0,
                    plus: result[0x04] & 0b0000_0100 == 0,
                    minus: result[0x04] & 0b0001_0000 == 0,
                    home: result[0x04] & 0b0000_1000 == 0,
                }
            }
            Self::NoConnection {} => WiiAccessoryReport::NoConnection {},
        }
    }
}
