use std::io;

extern crate cgbot;
use cgbot::debug::{self, Debug};

fn main() -> io::Result<()> {
    println!("KEK!");
    let mut debug = Debug::new();

    println!("LOL!");

    for i in 0..100 {
        debug.circle((50 * i) as f64, (50 * i) as f64, 50f64, debug::GREEN, true);
        debug.circle_popup((50 * i) as f64, (50 * i) as f64, 50f64, "kekeke");
        debug.end_frame();
    }

    Ok(())
}
