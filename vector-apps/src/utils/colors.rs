//! color conversion
//!
//! source: https://gist.github.com/andelf/1dcc50d1c9d4bd8ecf3151e6bafe87c0
//! via https://github.com/emgyrz/colorsys.rs which is not friendly to embedded devices
pub const RGB_UNIT_MAX: f32 = 255.0;
pub const HUE_MAX: f32 = 360.0;
pub const PERCENT_MAX: f32 = 100.0;
pub const RATIO_MAX: f32 = 1.0;
pub const ALL_MIN: f32 = 0.0;

const ONE: f32 = 1.0;
const TWO: f32 = 2.0;
const SIX: f32 = 6.0;

const ONE_THIRD: f32 = ONE / 3.0;
const TWO_THIRD: f32 = TWO / 3.0;

fn bound(r: f32, entire: f32) -> f32 {
    let mut n = r;
    loop {
        let less = n < ALL_MIN;
        let bigger = n > entire;
        if !less && !bigger {
            break n;
        }
        if less {
            n += entire;
        } else {
            n -= entire;
        }
    }
}

fn bound_ratio(r: f32) -> f32 {
    bound(r, RATIO_MAX)
}

fn calc_rgb_unit(unit: f32, temp1: f32, temp2: f32) -> u8 {
    let mut result = temp2;
    if SIX * unit < ONE {
        result = temp2 + (temp1 - temp2) * SIX * unit
    } else if TWO * unit < ONE {
        result = temp1
    } else if 3.0 * unit < TWO {
        result = temp2 + (temp1 - temp2) * (TWO_THIRD - unit) * SIX
    }
    (result * RGB_UNIT_MAX) as u8
}

pub fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    if s == 0.0 {
        let unit = (RGB_UNIT_MAX * l) as u8;
        return (unit, unit, unit);
    }

    let temp1 = if l < 0.5 {
        l * (ONE + s)
    } else {
        l + s - l * s
    };

    let temp2 = TWO * l - temp1;
    let hue = h;

    let temp_r = bound_ratio(hue + ONE_THIRD);
    let temp_g = bound_ratio(hue);
    let temp_b = bound_ratio(hue - ONE_THIRD);

    let r = calc_rgb_unit(temp_r, temp1, temp2);
    let g = calc_rgb_unit(temp_g, temp1, temp2);
    let b = calc_rgb_unit(temp_b, temp1, temp2);
    (r, g, b)
}
