use serial_ports::{ListPorts, ListPortInfo, ListPortType};
use serial::SerialPort;
use serial::core::Result as SerialResult;
use serial;

use super::pirate::BusPirate;

use std::path::PathBuf;
use std::cmp::Ordering;

const BUSPIRATE_VID: u16 = 0x04D8;
const BUSPIRATE_PID: u16 = 0xFB00;

const BUSPIRATE_SETTINGS: serial::PortSettings = serial::PortSettings {
    baud_rate:    serial::Baud115200,
    char_size:    serial::Bits8,
    parity:       serial::ParityNone,
    stop_bits:    serial::Stop1,
    flow_control: serial::FlowNone,
};

#[derive(Debug, Clone)]
pub struct Device {
    pub device: PathBuf,
    pub hwid: String
}

impl Device {
    pub fn open(&self) -> SerialResult<BusPirate> {
        let mut port = try!(serial::open(&self.device));
        try!(port.configure(&BUSPIRATE_SETTINGS));
        Ok(BusPirate::new(port))
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
                    .or_else(|| d.hwid.find(pat))
                    .is_some())
            .next()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn sort(&mut self) {
        self.0.sort_by(|ref a, ref b| a.device.cmp(&b.device));
    }

    pub fn sort_by<F>(&mut self, compare: F)
        where
        F: FnMut(&Device, &Device) -> Ordering {
        self.0.sort_by(compare)
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
