//use generated::CodesFromMCU;
use serialport::{self, SerialPort};
use std::{boxed::Box};
mod generated;

#[derive(Debug)]
pub enum MorpheusError  {
    FailedToEnumerate,
    PortNotFound,
    CannotOpenPort,
    NotImplemented
}

impl MorpheusError {
    pub fn to_string(&self) -> String {
        match &self {
            Self::CannotOpenPort => "Can't open port".into(),
            Self::FailedToEnumerate => "Failed to enumerate ports".into(),
            Self::PortNotFound => "Port not found".into(),
            Self::NotImplemented => "Not implemented".into()
        }
    }
}

/// Morpheus project serial port 
/// 
/// This structure is used to communicate with the Morpheus slave microcontroller.
/// 
/// Exemple:
/// ```rust,should_panic
/// use morpheus_serial::MorpheusSerial;
/// 
/// let serial = MorpheusSerial::new("/dev/ttyUSB0".into(), 115200).expect("Couldn't open port".into());
/// ```
pub struct MorpheusSerial {
    serial_rx: Box<dyn SerialPort>,
    serial_tx: Box<dyn SerialPort>
}

impl MorpheusSerial {
    /// Open a serial port to communicate with the MCU
    /// 
    /// Searches the serial port from known available serial port
    /// tries to open it.
    /// Returns a newly created serial port or a MorpheusError.
    /// 
    /// params:
    ///   - port: name of the serial port to open
    ///   - baudrate: communication speed (default should be 115200)
    pub fn new(port: String, baudrate: u32) -> Result<Self, MorpheusError> {
        if let Ok(ports) = serialport::available_ports() {
            if let Some(_found) = ports.iter().filter(|&p| {
                p.port_name == port
            }).next() {
                match serialport::new(port, baudrate).open() {
                    Ok(port) => Ok(Self { serial_rx: port.try_clone().expect("Failed to clone port"), serial_tx: port }),
                    Err(_e) => Err(MorpheusError::CannotOpenPort) 
                }
            }
            else {
                Err(MorpheusError::PortNotFound)
            }

        }
        else {
            Err(MorpheusError::FailedToEnumerate)
        }

    }

    /// Closes the serial port connection
    /// 
    /// NOTE: Does currently nothing except consuming "self",
    /// Rust frees data on its own.
    /// 
    pub fn close(self) {
        
    }

    //pub fn receive_frame(&self) -> Result<generated::CodesFromMCU, MorpheusError>  {
    //    Err(MorpheusError::NotImplemented)
    //}
}