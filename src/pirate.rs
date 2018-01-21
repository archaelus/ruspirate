use serial::{SerialPort, SystemPort, Error};
use serial::core::Result as SerialResult;
use serial;

#[derive(Debug)]
pub enum BinModeVSN {
    One
}
const BBIO_RESP_V1: [u8; 5] = [b'B', b'B', b'I', b'O', b'1'];

pub struct BusPirate {
    port: SystemPort
}

pub struct BBIOConn {
    port: SystemPort,
    pub vsn: BinModeVSN
}

use std::io::{Read, Write, ErrorKind};
use std::time::Duration;
impl BusPirate {
    pub fn new(port: SystemPort) -> Self {
        Self { port: port }
    }

    pub fn enter_bio_mode(self) -> SerialResult<BBIOConn> {
        let mut port = self.port;
        // Try to escape any prompt we're at.
        try!(write!(port, "\n\n\n\n\n\n\n\n\n\n#\n"));

        // Flush read
        let mut buffer: Vec<u8> = Vec::new();
        match port.read_to_end(&mut buffer) {
            Ok(_) => (),
            Err(ref e) if e.kind() == ErrorKind::TimedOut => (),
            Err(e) => return Err(e.into())
        }

        let original_timeout = port.timeout();
        try!(port.set_timeout(Duration::from_millis(20)));

        for _try in 1..40 {
            try!(port.write_all(&[0x00; 1]));
            let mut vsn_vec: [u8; 5] = [0; 5];
            match port.read_exact(&mut vsn_vec) {
                Err(ref e) if e.kind() == ErrorKind::TimedOut => continue,
                Err(e) => {
                    try!(port.set_timeout(original_timeout));
                    return Err(e.into())
                }
                Ok(()) => {
                    if vsn_vec == BBIO_RESP_V1 {
                        try!(port.set_timeout(original_timeout));
                        return Ok(BBIOConn { port: port,
                                             vsn: BinModeVSN::One })
                    }
                    try!(port.set_timeout(original_timeout));
                    return Err(Error::new(serial::ErrorKind::InvalidInput,
                                          format!("Got {:?} while entering binmode",
                                                  vsn_vec)));
                }
            }
        }
        try!(port.set_timeout(original_timeout));
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
