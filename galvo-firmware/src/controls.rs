use vector_apps::apps::Controls;
use wii_accessories::WiiAccessoryReport;

pub fn parse_report(report: WiiAccessoryReport) -> Controls {
    match report {
        WiiAccessoryReport::Nunchuck { joy, c, z, .. } => Controls {
            x: joy.x,
            y: joy.y,
            a: c,
            b: z,
        },
        WiiAccessoryReport::ClassicController { left, a, b, .. } => Controls {
            x: left.x,
            y: left.y,
            a,
            b,
        },
        WiiAccessoryReport::NoConnection {} => Controls {
            x: 0,
            y: 0,
            a: false,
            b: false,
        },
    }
}
