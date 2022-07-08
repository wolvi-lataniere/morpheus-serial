use generated::Feedbacks;
//use generated::CodesFromMCU;
use tokio_serial::{self, SerialPort, SerialStream};
use std::io::ErrorKind;
use tokio;
use tokio::task;
use tokio::sync::{mpsc, broadcast};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::time::Duration;

pub mod generated;


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
    pub tx: tokio::sync::broadcast::Sender<i32>,
    pub rx_queue: tokio::sync::broadcast::Receiver<Feedbacks>,
    tx_chan: mpsc::Sender<Vec<u8>>
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
    pub fn new(port: String, baudrate: u32) -> Result<(Self, task::JoinHandle<Result<(), MorpheusError>>), MorpheusError> {
        match SerialStream::open(&tokio_serial::new(port, baudrate).timeout(Duration::from_millis(10))) {
            Ok(mut port) => {
                port.set_exclusive(true).unwrap();
                let (tx, rx) = tokio::sync::broadcast::channel(4);
                let (tx_chan, rx_chan) = mpsc::channel::<Vec<u8>>(10);
                let (tx_feedback, rx_feedback) = broadcast::channel(5);

                let rx_task = Self::receive_frame(port, rx, rx_chan, tx_feedback);
                Ok((Self { tx_chan, tx, rx_queue: rx_feedback}, rx_task))
            },
            Err(_e) => Err(MorpheusError::CannotOpenPort) 
        }
    }

    /// Closes the serial port connection
    /// 
    /// NOTE: Does currently nothing except consuming "self",
    /// Rust frees data on its own.
    /// 
    pub async fn close(self) {

    }

    pub async fn send_frame(&self, inst: generated::Instructions) -> Result<(), MorpheusError> {
        let content = inst.to_bytes();
        let csum = content.iter().fold(content.len()+4, |a, v| (a+(*v as usize))&0xFF) as u8;
        let frame = vec![0x55u8, 0xaau8, (content.len() as u8) + 4u8];

        let frame = [frame, content, vec![csum]].concat();
        println!("Sending Instruction: {}", frame.iter().map(|a| format!("{:02x} ", a)).collect::<Vec<String>>().join(""));
        self.tx_chan.send(frame).await.expect("Failed to send message");
        Ok(())
    }

    fn receive_frame(mut serial: SerialStream, mut rx: tokio::sync::broadcast::Receiver<i32>, mut rx_chan: mpsc::Receiver<Vec<u8>>, mut tx_feedback: broadcast::Sender<Feedbacks>) -> task::JoinHandle<Result<(), MorpheusError>>  {
        serial.clear(tokio_serial::ClearBuffer::Input).unwrap();
        serial.write_data_terminal_ready(true).unwrap();

        let mut buffer: Vec<u8> = vec![];
        task::spawn(async move {
            let mut state = FrameReceptionStats::Idle;
            let mut size = 0u8;
            let mut csum = 0u8;

            loop {
                let mut bytes = [0u8; 256];
            
                tokio::select!(
                    _ = rx.recv() => { 
                        println!("Quit request received");
                        serial.write_data_terminal_ready(false).unwrap();
                        return Ok(()); },
                    data = rx_chan.recv() => {
                        serial.write_all(data.unwrap().as_slice()).await.unwrap();
                    },
                    read = serial.read(&mut bytes) => { match read {
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
                                            tx_feedback.send(fb).unwrap();
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
                    }});
            }
        })
    }
}