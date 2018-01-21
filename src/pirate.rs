use serial::{SerialPort, SystemPort, Error};
use serial::core::Result;
use serial;

const BBIO_RESP_V1: [u8; 5] = [b'B', b'B', b'I', b'O', b'1'];

pub struct BusPirate {
    port: SystemPort
}

use std::io::{Read, Write, ErrorKind};
use std::time::{Instant, Duration};

use super::bbio::{BBIOConn, BinModeVSN};

impl BusPirate {
    pub fn new(port: SystemPort) -> Self {
        Self { port: port }
    }

    pub fn read_vsn(&mut self) -> Result<String> {
        let original_timeout = self.port.timeout();
        try!(self.port.set_timeout(Duration::from_millis(100)));
        try!(write!(self.port, "\n\n\n\n\n\n\n\n\n\n#\n"));
        let mut boot_str: String = String::new();
        let start = Instant::now();
        loop {
            if start.elapsed() > Duration::from_secs(5) {
                break;
            }

            let mut this_read: [u8; 1] = [0; 1];
            match self.port.read_exact(&mut this_read) {
                Ok(()) => {
                    let s = String::from_utf8_lossy(&this_read);
                    boot_str.push_str(&s);
                }
                Err(ref e) if e.kind() == ErrorKind::TimedOut => break,
                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => {
                    try!(self.port.set_timeout(original_timeout));
                    return Err(e.into())
                }
            }
        }
        try!(self.port.set_timeout(original_timeout));
        // Clean up the output (remove response to state reset string,
        // trailing prompt)
        Ok(boot_str.split("\r\n")
           .skip_while(|s| s != &"RESET")
           .skip(1)
           .filter(|s| ! s.starts_with("HiZ>"))
           .collect::<Vec<&str>>()
           .join("\n"))
    }

    pub fn enter_bio_mode(self) -> Result<BBIOConn> {
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
                        return Ok(BBIOConn::new(port, BinModeVSN::One))
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
