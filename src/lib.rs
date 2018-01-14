#![feature(conservative_impl_trait)]
extern crate serial_ports;

extern crate serial;

use serial_ports::{ListPorts, ListPortInfo, ListPortType};
use serial::{SerialPort, SystemPort};
use serial::core::Result as SerialResult;

const BUSPIRATE_VID: u16 = 0x04D8;
const BUSPIRATE_PID: u16 = 0xFB00;

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Device {
    pub device: PathBuf,
    pub hwid: String
}

const BUSPIRATE_SETTINGS: serial::PortSettings = serial::PortSettings {
    baud_rate:    serial::Baud115200,
    char_size:    serial::Bits8,
    parity:       serial::ParityNone,
    stop_bits:    serial::Stop1,
    flow_control: serial::FlowNone,
};

impl Device {
    pub fn open(&self) -> SerialResult<SystemPort> {
        let mut port = try!(serial::open(&self.device));
        try!(port.configure(&BUSPIRATE_SETTINGS));
        Ok(port)
    }
}

pub struct Devices(Vec<Device>);

impl Devices {
    pub fn detect() -> Devices {
        Devices (
            ListPorts::new().iter()
                .filter(is_bus_pirate)
                .map(|port|
                     Device {
                         device: port.device.clone(),
                         hwid: port.hwid.clone()
                     })
                .collect()
        )
    }

    pub fn default(&self) -> Option<&Device> {
        self.0.first()
    }

    pub fn find(&self, pat: &str) -> Option<&Device> {
        self.0.iter()
            .filter(|d| d.device.to_str()
                    .and_then(|dev| dev.find(pat))
                    .is_some())
            .next()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl IntoIterator for Devices {
    type Item = Device;
    type IntoIter = ::std::vec::IntoIter<Device>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

fn is_bus_pirate(port: &&ListPortInfo) -> bool {
    match port.port_type {
        ListPortType::UsbPort(ref usbport) =>
            usbport.vid == BUSPIRATE_VID && usbport.pid == BUSPIRATE_PID,
        _ =>
            false
    }
}
