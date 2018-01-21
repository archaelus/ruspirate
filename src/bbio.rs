use serial::{SystemPort, Error, ErrorKind};
use serial::core::Result;
use std::io::{Read, Write};
use super::i2c::I2CConn;

#[derive(Debug)]
pub enum BinModeVSN {
    One
}

pub struct BBIOConn {
    port: SystemPort,
    pub vsn: BinModeVSN
}

impl BBIOConn {
    pub fn new(port: SystemPort, vsn: BinModeVSN) -> Self {
        Self { port: port, vsn: vsn }
    }

    pub fn enter_i2c_mode(self) -> Result<I2CConn> {
        let mut port = self.port;
        let msg = Message::I2C;
        try!(port.write_all(&msg.send()));
        let good_reply = msg.expect();
        use std::iter;
        let mut buf = iter::repeat::<u8>(0)
            .take(good_reply.len()).collect::<Vec<u8>>();
        try!(port.read_exact(&mut buf));
        if buf == good_reply {
            return Ok(I2CConn::new(port))
        }
        Err(Error::new(ErrorKind::InvalidInput,
                       "couldn't enter binary I2C mode"))
    }
}

// Main BP/IO Messages
//
// 00000000 - Reset, responds "BBIO1"
// This command resets the Bus Pirate into raw bitbang mode from the
// user terminal. It also resets to raw bitbang mode from raw SPI
// mode, or any other protocol mode. This command always returns a
// five byte bitbang version string "BBIOx", where x is the current
// protocol version (currently 1).  Some terminals send a NULL
// character (0x00) on start-up, causing the Bus Pirate to enter
// binary mode when it wasn't wanted. To get around this, you must now
// enter 0x00 at least 20 times to enter raw bitbang mode.  Note: The
// Bus Pirate user terminal could be stuck in a configuration menu
// when your program attempts to enter binary mode. One way to ensure
// that you're at the command line is to send <enter> at least 10
// times, and then send '#' to reset. Next, send 0x00 to the command
// line 20+ times until you get the BBIOx version string.
// After entering bitbang mode, you can enter other binary protocol modes.
//
// 00000001 - Enter binary SPI mode, responds "SPI1"
// Binary SPI mode is documented here
//
// 00000010 - Enter binary I2C mode, responds "I2C1"
// Binary I2C mode is documented here.
//
// 00000011 - Enter binary UART mode, responds "ART1"
// Binary UART mode is documented here.
//
// 00000100 - Enter binary 1-Wire mode, responds "1W01"
// Binary 1-Wire mode is documented here.
//
// 00000101 - Enter binary raw-wire mode, responds "RAW1"
// Binary raw-wire mode is documented here.
//
// 00000110 - Enter OpenOCD JTAG mode
// OpenOCD mode is documented in the source only.
//
// 0000xxxx - Reserved for future raw protocol modes
//
// 00001111 - Reset Bus Pirate
// The Bus Pirate responds 0x01 and then performs a complete hardware
// reset. The hardware and firmware version is printed (same as the
// 'i' command in the terminal), and the Bus Pirate returns to the
// user terminal interface. Send 0x00 20 times to enter binary mode
// again.
// Note: there may be garbage data between the 0x01 reply and the
// version information as the PIC UART initializes.
//
// 0001000x - Bus Pirate self-tests
// Binary self tests are documented here. Available only in v2go and
// v3 hardware.
//
// 00010010 - Setup pulse-width modulation (requires 5 byte setup)
// Configure and enable pulse-width modulation output in the AUX
// pin. Requires a 5 byte configuration sequence. Responds 0x01 after
// a complete sequence is received. The PWM remains active after
// leaving binary bitbang mode!  Equations to calculate the PWM
// frequency and period are in the PIC24F output compare manual. Bit 0
// and 1 of the first configuration byte set the prescaler value. The
// Next two bytes set the duty cycle register, high 8bits first. The
// final two bytes set the period register, high 8bits first.
//
// 00010011 - Clear/disable PWM
// Clears the PWM, disables PWM output. Responds 0x01.
//
// 00010100 - Take voltage probe measurement (returns 2 bytes)
// Take a measurement from the Bus Pirate voltage probe. Returns a 2
// byte ADC reading, high 8bits come first. To determine the actual
// voltage measurement: (ADC/1024)*3.3volts*2; or simply
// (ADC/1024)*6.6.
//
// 00010101 - Continuous voltage probe measurement
//
// Sends ADC data (2bytes, high 8 first) as fast as UART will allow. A
// new reading is not taken until the previous finishes transmitting
// to the PC, this prevents time distortion from the buffer. Added for
// the oscilloscope script.
//
// 00010110 - Frequency measurement on AUX pin
// Takes frequency measurement on AUX pin. Returns 4byte frequency
// count, most significant byte first.
//
// 010xxxxx - Configure pins as input(1) or output(0): AUX|MOSI|CLK|MISO|CS
// Configure pins as an input (1) or output(0). The pins are mapped to
// the lower five bits in this order:
// AUX|MOSI|CLK|MISO|CS.
//
// The Bus pirate responds to each direction update with a byte
// showing the current state of the pins, regardless of
// direction. This is useful for open collector I/O modes.
//
// 1xxxxxxx - Set on (1) or off (0): POWER|PULLUP|AUX|MOSI|CLK|MISO|CS
// The lower 7bits of the command byte control the Bus Pirate pins and
// peripherals. Bitbang works like a player piano or bitmap. The Bus
// Pirate pins map to the bits in the command byte as follows:
// 1|POWER|PULLUP|AUX|MOSI|CLK|MISO|CS
// The Bus pirate responds to each update with a byte in the same
// format that shows the current state of the pins.

pub enum Message {
    ResetProto,
    SPI,
    I2C,
    UART,
    OneWire,
    RawWire,
    OpenOCDJTAG,
    Reserved(u8),
    ResetDevice,
    SelfTest,
    SetupPWM,
    DisablePWM,
    ProbeVoltage,
    ContinuousVoltage,
    MeasureFrequency,
    ConfigurePinIO(bool, Pin),
    SetOnOff(bool, OnOffItem)
}

pub enum Pin {
    AUX,
    MOSI,
    CLK,
    MISO,
    CS
}

pub enum OnOffItem {
    AUX,
    MOSI,
    CLK,
    MISO,
    CS,
    Power,
    Pullup
}

use self::Message::*;
impl Message {
    pub fn send(&self) -> Vec<u8> {
        match *self {
            ResetProto => vec![0b00000000],
            SPI  => vec![0b00000001],
            I2C  => vec![0b00000010],
            UART => vec![0b00000011],
            OneWire => vec![0b00000100],
            RawWire => vec![0b00000101],
            OpenOCDJTAG => vec![0b00000110],
            Reserved(b) => vec![0b00001111 & b],
            ResetDevice => vec![0b00001111],
            _ => unimplemented!()
        }
    }

    pub fn expect(&self) -> Vec<u8> {
        match *self {
            SPI  => vec![b'S', b'P', b'I', b'1'],
            I2C  => vec![b'I', b'2', b'C', b'1'],
            UART => vec![b'A', b'R', b'T', b'1'],
            OneWire => vec![b'1', b'W', b'0', b'1'],
            RawWire => vec![b'R', b'A', b'W', b'1'],
            _ => unimplemented!()
        }
    }
}
