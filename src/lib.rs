#![feature(conservative_impl_trait)]
extern crate serial_ports;
extern crate serial;

use serial_ports::{ListPorts, ListPortInfo, ListPortType};
use serial::{SerialPort, SystemPort, Error};
use serial::core::Result as SerialResult;

const BUSPIRATE_VID: u16 = 0x04D8;
const BUSPIRATE_PID: u16 = 0xFB00;

use std::path::PathBuf;
use std::cmp::Ordering;

#[derive(Debug)]
pub enum BinModeVSN {
    One
}
const BBIO_RESP_V1: [u8; 5] = [b'B', b'B', b'I', b'O', b'1'];

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
        Ok(BusPirate { port: port })
    }
}

pub struct BusPirate {
    port: SystemPort
}

use std::io::{Read, Write, ErrorKind};
use std::time::Duration;
impl BusPirate {
    pub fn test(&mut self) -> SerialResult<BinModeVSN> {
        // Try to escape any prompt we're at.
        try!(write!(&mut self.port, "\n\n\n\n\n\n\n\n\n\n#\n"));
        try!(self.flush_read());
        self.enter_bio_mode()
    }

    fn flush_read(&mut self) -> SerialResult<()> {
        let mut buffer: Vec<u8> = Vec::new();
        match self.port.read_to_end(&mut buffer) {
            Ok(_) => Ok(()),
            Err(ref e) if e.kind() == ErrorKind::TimedOut => Ok(()),
            Err(e) => Err(e.into())
        }
    }

    fn enter_bio_mode(&mut self) -> SerialResult<BinModeVSN> {
        let original_timeout = self.port.timeout();
        try!(self.port.set_timeout(Duration::from_millis(20)));

        for _try in 1..40 {
            try!(self.port.write_all(&[0x00; 1]));
            let mut vsn_vec: [u8; 5] = [0; 5];
            match self.port.read_exact(&mut vsn_vec) {
                Err(ref e) if e.kind() == ErrorKind::TimedOut => continue,
                Err(e) => {
                    try!(self.port.set_timeout(original_timeout));
                    return Err(e.into())
                }
                Ok(()) => {
                    if vsn_vec == BBIO_RESP_V1 {
                        try!(self.port.set_timeout(original_timeout));
                        return Ok(BinModeVSN::One)
                    }
                    try!(self.port.set_timeout(original_timeout));
                    return Err(Error::new(serial::ErrorKind::InvalidInput,
                                          format!("Got {:?} while entering binmode",
                                                  vsn_vec)));
                }
            }
        }
        try!(self.port.set_timeout(original_timeout));
        Err(Error::new(serial::ErrorKind::InvalidInput,
                       "couldn't enter binary IO mode"))
    }
}

use std::fmt;
use serial::unix::TTYPort;
use std::os::unix::io::AsRawFd;
impl fmt::Debug for BusPirate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let port: &TTYPort = &self.port;
        write!(f, "BusPirate {{ port: {} }}", port.as_raw_fd())
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
