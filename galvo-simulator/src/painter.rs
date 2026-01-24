use std::{
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
        mpsc::Sender,
    },
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use vector_apps::{
    apps::{
        VectorApp,
        alphabet::AlphabetDemo,
        asteroids::Asteroids,
        clock::{Clock, TimeSource},
        cube::CubeDemo,
        cycle::Cycle,
        maps::Maps,
    },
    point::Point,
};

pub struct SystemTimeSource;
impl TimeSource for SystemTimeSource {
    fn now(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};

        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

pub fn painter(tx: Sender<Point>) {
    let mut app = Cycle::new(vec![
        // Box::new(AlphabetDemo::new()),
        // Box::new(CubeDemo::new()),
        // Box::new(Asteroids::new()),
        // Box::new(Maps::new()),
        Box::new(Clock::new(SystemTimeSource)),
    ]);

    let mut frame = 0;
    loop {
        let path = app.get_path(frame);

        for point in path {
            tx.send(*point).unwrap();
            thread::sleep(Duration::from_micros(point.delay as u64));
        }

        frame += 1;
    }
}
