use std::io;
use std::io::Write;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let mut i = 0;
    loop {
        sleep(Duration::from_secs(1));
        println!("{} Starting game server...", i);
        i += 1;
        io::stdout().flush().unwrap();
    }
}