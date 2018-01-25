#[macro_use]
extern crate clap;

extern crate ruspirate;

use ruspirate::{Devices};
use ruspirate::i2c::{PullUp, Speed, BusSettings};
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {

    let matches = clap_app!(Rpir8 =>
                            (@setting SubcommandRequiredElseHelp)
                            (version: VERSION)
                            (author: "Geoff Cant <geoff+rust@archant.us>")
                            (about: "Bus pirates things. With Rust!")
                            (@subcommand list =>
                             (about: "List buspirates"))
                            (@subcommand test =>
                             (about: "Test a buspirate")
                             (@arg dev: -d --dev +takes_value
                              "The bus pirate device to use."))
                            (@subcommand vsn =>
                             (about: "Interrogate the version of the buspirate")
                             (@arg dev: -d --dev +takes_value
                              "The bus pirate device to use."))
                            (@subcommand i2c =>
                             (about: "I2C commands")
                             (@arg dev: -d --dev +takes_value
                              "The bus pirate device to use.")
                             (@arg voltage: -v --voltage
                              +takes_value
                              "The bus voltage to use. (5, 3.3, 0 if not specified)")
                             (@arg speed: -s --speed
                              +takes_value
                              "The bus speed to use (in Hz). (400k, 100k, 50k, 5k)")
                             (@arg dryrun: -r --("dry-run")
                              "Don't actually execute the command.")
                             (@subcommand scan =>
                              (about: "Scan the i2c bus for r/w addresses"))
                             (@subcommand test =>
                              (about: "Test setting up binary i2c mode"))
                            )
    ).get_matches();

    let pirates = Devices::detect();

    match matches.subcommand_name() {
        Some("list") => {
            match pirates.len() {
                0 => {
                    println!("No bus pirates found.");
                    std::process::exit(1);
                }
                _ => {
                    for (i, p) in pirates.into_iter().enumerate() {
                        println!("({}) {dev:?} ({hwid})",
                                 i+1, dev=p.device, hwid=p.hwid);
                    }
                }
            }
        },
        Some("test") => {
            let test = matches.subcommand_matches("test").unwrap();

            let device = pirates.find_or_default(test.value_of("dev"));

            match device {
                None => {
                    println!("No bus pirate found.");
                    std::process::exit(1);
                },
                Some(pirate) => {
                    println!("Testing {:?}", pirate);
                    match pirate.open() {
                        Ok(mut p) => {
                            println!("Yay! Opened {:?} as {:#?}",
                                     pirate.device.to_str(), p);
                            match p.enter_bio_mode() {
                                Err(e) => println!("Testing failed: {:#?}", e),
                                Ok(c) => println!("Good bbio con {:?}!",
                                                  c.vsn)
                            }
                        },
                        Err(e) => {
                            println!("Couldn't open {:?}: {}",
                                     pirate.device.to_str(), e);
                            std::process::exit(1);
                        }
                    }
                }
            }
        },
        Some("vsn") => {
            let device =
                pirates.find_or_default(matches.subcommand_matches("vsn")
                                        .unwrap().value_of("dev"))
                .expect("Couldn't find a bus pirate device.");

            device.open()
                .expect("Couldn't open bus_pirate")
                .read_vsn()
                .and_then(|s| Ok(println!("{}:\n{}",
                                          device.device.to_str().unwrap(), s)))
                .expect("Couldn't get version string.");
        },
        Some("i2c") => {
            let i2c_matches = matches.subcommand_matches("i2c").unwrap();
            let dev = pirates.find_or_default(i2c_matches.value_of("dev"));
            let voltage = value_t!(i2c_matches, "voltage", PullUp);
            let speed = value_t!(i2c_matches, "speed", Speed)
                .unwrap_or(Speed::Hz100000);
            let dryrun: bool = value_t!(i2c_matches, "dryrun", bool)
                .unwrap_or(true);
            println!("I2C: dev: {:?} voltage: {:?} speed: {:?} dryrun: {:?}",
                     dev, voltage, speed, dryrun);

            match i2c_matches.subcommand_name() {
                Some("scan") => {},
                Some("test") => {
                    let mut i2c = dev.expect("Couldn't find a bus_pirate")
                        .open()
                        .expect("Couldn't open bus_pirate")
                        .enter_bio_mode()
                        .expect("Couldn't enter binary IO mode")
                        .enter_i2c_mode()
                        .expect("Couldn't enter binary I2C mode");

                    i2c.configure(&BusSettings::new(speed, !dryrun, false, false))
                        .expect("Couldn't configure the I2C bus");
                    println!("Configured! Yay!");
                    i2c.test()
                        .expect("Failed to get I2C vsn.");
                    println!("I guess that worked! Yay!");
                }
                Some(ref c) => {
                    println!("Unknown i2c command: {}", c);
                    std::process::exit(1);
                }
                None => {
                    println!("I2C: no i2c command to run.");
                }
            }
        },
        _ => {
            println!("Unknown subcommand.");
            std::process::exit(1);
        }
    }

    std::process::exit(0);
}
