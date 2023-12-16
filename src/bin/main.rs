use std::io;

use cgbot::rewind_client::RewindClient;

fn main() -> io::Result<()> {
    println!("KEK!");
    let mut client = RewindClient::with_host_port("192.168.0.100", 9111)?;

    println!("LOL!");

    for i in 0..10 {
        client.circle(
            (50 * i) as f64,
            (50 * i) as f64,
            50f64,
            RewindClient::GREEN,
            true,
        )?;
        client.end_frame()?;
    }

    Ok(())
}
