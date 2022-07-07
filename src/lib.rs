//use generated::CodesFromMCU;
use tokio_serial::{self, SerialPort};
use std::io::ErrorKind;
use std::{boxed::Box};
pub mod generated;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
pub enum MorpheusError  {
    FailedToEnumerate,
    PortNotFound,
    CannotOpenPort,
    FailedToRead,
    FailedToWrite,
    NotImplemented
}

impl MorpheusError {
    pub fn to_string(&self) -> String {
        match &self {
            Self::CannotOpenPort => "Can't open port".into(),
            Self::FailedToEnumerate => "Failed to enumerate ports".into(),
            Self::FailedToRead => "Failed to read".into(),
            Self::FailedToWrite => "Failed to write".into(),
            Self::PortNotFound => "Port not found".into(),
            Self::NotImplemented => "Not implemented".into()
        }
    }
}

enum FrameReceptionStats {
    Idle,
    Header2,
    HeaderOk,
    Receiving,
    WaitCSum
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
    serial: Box<dyn SerialPort>
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
        match tokio_serial::new(port, baudrate).timeout(Duration::from_millis(500)).open() {
            Ok(port) => Ok(Self { serial: port }),
            Err(_e) => Err(MorpheusError::CannotOpenPort) 
        }
    }

    /// Closes the serial port connection
    /// 
    /// NOTE: Does currently nothing except consuming "self",
    /// Rust frees data on its own.
    /// 
    pub fn close(self) {
        
    }

    pub fn send_frame(&mut self, inst: generated::Instructions) -> Result<(), MorpheusError> {
        let content = inst.to_bytes();
        let csum = content.iter().fold(content.len()+4, |a, v| (a+(*v as usize))&0xFF) as u8;
        let frame = vec![0x55u8, 0xaau8, (content.len() as u8) + 4u8];

        let frame = [frame, content, vec![csum]].concat();
        println!("Sending Instruction: {}", frame.iter().map(|a| format!("{:02x} ", a)).collect::<Vec<String>>().join(""));
        self.serial.write_all(frame.as_slice()).unwrap();
        Ok(())
    }

    pub fn receive_frame(&self) -> thread::JoinHandle<Result<generated::Feedbacks, MorpheusError>>  {
        let mut serial = self.serial.try_clone().unwrap();
        serial.clear(tokio_serial::ClearBuffer::Input).unwrap();
        serial.write_request_to_send(true).unwrap();
        let mut buffer: Vec<u8> = vec![];
        thread::spawn(move || {
            let mut state = FrameReceptionStats::Idle;
            let mut size = 0u8;
            let mut csum = 0u8;
            loop {
            let mut bytes = [0u8; 256];
            match serial.read(&mut bytes) {
                Ok(amount) => {
                    println!("Receiving: {}", bytes[..amount].iter().map(|d| format!("{:02x} ", d)).collect::<Vec<String>>().join(""));
                   
                    for ele in bytes[..amount].iter() {
                        match state {
                            FrameReceptionStats::Idle => {
                                if *ele == 0x55u8 {
                                    state = FrameReceptionStats::Header2;
                                }
                            },
                            FrameReceptionStats::Header2 => {
                                if *ele == 0xAAu8 {
                                    state = FrameReceptionStats::HeaderOk;
                                } else {
                                    state = FrameReceptionStats::Idle;
                                }
                            },
                            FrameReceptionStats::HeaderOk => {
                                buffer.clear();
                                size = *ele;
                                csum = *ele;
                                state = FrameReceptionStats::Receiving;
                            },
                            FrameReceptionStats::Receiving => {
                                buffer.push(*ele);
                                csum = csum + *ele;
                                if (buffer.len() + 4) >= size as usize {
                                    state = FrameReceptionStats::WaitCSum;
                                }
                            },
                            FrameReceptionStats::WaitCSum => {
                                if csum == *ele {
                                    let fb = generated::Feedbacks::from_bytes(&buffer).unwrap();
                                    println!("Received Feedback: {:?}", fb);
                                } else {
                                    println!("Failed parsing, {} != {}", ele, csum);
                                }

                                state = FrameReceptionStats::Idle;
                            }
                        }
                    }
                    //let fb = generated::Feedbacks::from_bytes(&bytes[..amount]).unwrap();
                 
                },
                Err(e) => {if ErrorKind::TimedOut != e.kind() { println!("{:?}", e) }} //{return Err(MorpheusError::FailedToRead);}
            }
            
        }})
    }
}