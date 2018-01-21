#![feature(conservative_impl_trait)]
extern crate serial_ports;
extern crate serial;

mod device;
mod pirate;

pub use pirate::BusPirate;
pub use device::{Device, Devices};
