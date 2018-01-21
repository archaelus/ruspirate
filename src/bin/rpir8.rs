#[macro_use]
extern crate clap;

extern crate ruspirate;

use ruspirate::{Devices};
use ruspirate::i2c::{PullUp, Speed};
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
                              "The bus voltage to use. (5, 3.3, 0 if not specified)")
                             (@arg dryrun: -r --("dry-run")
                              "Don't actually execute the command.")
                             (@subcommand scan =>
                              (about: "Scan the i2c bus for r/w addresses"))
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

            let device = match test.value_of("dev") {
                None => pirates.default(),
                Some(pat) => pirates.find(pat)
            };

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
                default_device(&pirates,
                               matches.subcommand_matches("vsn")
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
            let dev = default_device(&pirates, i2c_matches.value_of("dev"));
            let voltage = value_t!(i2c_matches, "voltage", PullUp);
            let speed = value_t!(i2c_matches, "speed", Speed);
            let dryrun: bool = value_t!(i2c_matches, "dryrun", bool)
                .unwrap_or(true);
            println!("I2C: dev: {:?} voltage: {:?} speed: {:?} dryrun: {:?}",
                     dev, voltage, speed, dryrun);

            match i2c_matches.subcommand_name() {
                Some("scan") => {},
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

use ruspirate::Device;
fn default_device<'a>(devs: &'a Devices, dev: Option<&str>) ->
    Option<&'a Device> {
    dev.map_or_else(|| devs.default(),
                    |pat| devs.find(pat))
}
