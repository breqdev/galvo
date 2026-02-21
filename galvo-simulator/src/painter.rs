use std::{
    sync::mpsc::{Receiver, Sender},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use vector_apps::{
    apps::{
        Controls, VectorApp,
        alphabet::AlphabetDemo,
        asteroids::Asteroids,
        clock::{Clock, TimeSource},
        cube::CubeDemo,
        cycle::Cycle,
        ilda::Ilda,
        maps::Maps,
        mbta::Mbta,
    },
    point::Point,
};

pub struct SystemTimeSource;
impl TimeSource for SystemTimeSource {
    fn now(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

pub fn painter(tx: Sender<Point>, rx: Receiver<Controls>) {
    let mut app = Cycle::new(vec![
        Box::new(AlphabetDemo::new("LITT.CHR Aa1!".to_string())),
        // Box::new(CubeDemo::new()),
        // Box::new(Asteroids::new()),
        // Box::new(Maps::new()),
        // Box::new(Ilda::new()),
        // Box::new(Mbta::new()),
        // Box::new(Clock::new(SystemTimeSource)),
    ]);

    let mut frame = 0;
    loop {
        let path = app.get_path(frame);

        for point in path {
            tx.send(*point).unwrap();
            thread::sleep(Duration::from_micros(point.delay as u64));
        }

        while let Ok(controls) = rx.try_recv() {
            app.handle_controls(controls);
        }

        frame += 1;
    }
}
