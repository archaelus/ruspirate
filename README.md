# ruspirate
[Buspirate](http://dangerousprototypes.com/docs/Bus_Pirate) Crate for Rust

Allows you to detect, open, and control a Bus Pirate from rust. Some utilities are included to read and write eeproms with it.

## Prerequisites

I use nightly rust (`rustc 1.25.0-nightly (e6072a7b3 2018-01-13)`) for now and haven't tested other versions.

## Install and Build

    $ git clone https://github.com/archaelus/ruspirate.git
    $ cargo build

## Utils

### List Pirates

List attached bus pirates.

    $ cargo run --bin list_pirates

### rpir8

The swiss army knife of rust-controlled buspirate activity.

    $ cargo run --bin rpir8
       Compiling ruspirate v0.1.0 (file:///....../rust/ruspirate)
        Finished dev [unoptimized + debuginfo] target(s) in 1.43 secs
         Running `target/debug/rpir8`
    Rpir8 0.1.0
    Geoff Cant <geoff+rust@archant.us>
    Bus pirates things. With Rust!
    
    USAGE:
        rpir8 <SUBCOMMAND>
    
    FLAGS:
        -h, --help       Prints help information
        -V, --version    Prints version information
    
    SUBCOMMANDS:
        help    Prints this message or the help of the given subcommand(s)
        list    List buspirates
        test    Test a buspirate

List attached bus pirates:

    $ cargo run --bin=rpir8 list
    (1) "/dev/cu.usbmodem00000001" (USB VID:PID=04D8:FB00 SER=00000001 LOCATION=20-1.3.4.1)

Test the default attached bus pirate:

    $ cargo run --bin=rpir8 test
    Testing Device { device: "/dev/cu.usbmodem00000001", hwid: "USB VID:PID=04D8:FB00 SER=00000001 LOCATION=20-1.3.4.1" }
    Yay! Opened Some("/dev/cu.usbmodem00000001") as BusPirate { port: 3 }
    Good pirate (vsn One)!

Grab the reset version information of the bus pirate:

    $ cargo run --bin=rpir8 vsn -- -d /dev/cu.usbmodem
    /dev/cu.usbmodem00000001:
    Bus Pirate v4
    Firmware v6.2-beta1 r1981 
    DEVID:0x1019 REVID:0x0004 (24FJ256GB106 UNK)
    http://dangerousprototypes.com
