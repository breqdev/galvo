use std::{sync::mpsc::Sender, thread, time::Duration};

use vector_apps::{
    apps::{VectorApp, alphabet::AlphabetDemo, cube::CubeDemo},
    point::Point,
};

pub fn painter(tx: Sender<Point>) {
    let mut app: Box<dyn VectorApp> = Box::new(CubeDemo::new());
    // let app: Box<dyn VectorApp> = Box::new(AlphabetDemo::new());

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
