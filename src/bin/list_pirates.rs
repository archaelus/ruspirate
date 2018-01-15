extern crate serial_ports;
extern crate ruspirate;

use ruspirate::{Devices};

fn main() {
    let mut pirates = Devices::detect();
    pirates.sort();

    for pirate in pirates {
        println!("Bus pirate: {:?}",
                 pirate)
    }
}
