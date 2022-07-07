use std::ops::Range;
use std::vec;
use std::{process::exit, time::Duration, thread};

use morpheus_serial::MorpheusSerial;
use morpheus_serial::generated;
use tokio_serial;
use getopt::Opt;

fn list_serial_ports() -> Result<(), tokio_serial::Error>{
    let available = tokio_serial::available_ports()?;

    println!("{} available serial ports:", available.len());
    available.iter().for_each(|port| {
       println!("\t- {:?}: {}", port.port_type, port.port_name);
    });

    Ok(())
}

fn display_help(name: &String) {
    println!("Usage:");
    println!("\t{} -p PORT_NAME [-b BAUDRATE]", name);
    println!();
    println!("Parameters:");
    println!("\t-h Display this help");
    println!("\t-l List available serial ports");
    println!("\t-p PORT_NAME: name of the serial port to open (e.g. /dev/ttyAMA0)");
    println!("\t-b BAUDRATE: communication baudrate in bits per second (default 115200)"); 
}

fn main() {
    println!("Morpheus serial server");

    let args: Vec<String> = std::env::args().collect();
    let mut opts = getopt::Parser::new(&args, "lp:b:h");

    let mut port : Option<String> = None;
    let mut baudrate : u32 = 115200;

    loop {
        match opts.next().transpose().unwrap() {
            None => break,
            Some(opt) => match opt {
                Opt('l', None) => {list_serial_ports().unwrap_or(()); exit(0)},
                Opt('b', Some(string)) => {
                    baudrate = u32::from_str_radix(string.as_str(), 10).expect("Wrong baudrate format, expecting a number");
                    println!("Setting baudrate to: {}", baudrate);
                },
                Opt('p', Some(string)) => {
                    port = Some(string);
                },
                Opt('h', None) => {
                    display_help(&args.first().unwrap());
                    exit(0);
                }
                _ => unreachable!(),
            }
        }
    }

    if port.is_none() {
        display_help(&args.first().unwrap());
        eprintln!("Error: Port must be set!");
        exit(1);
    }

    println!("Openning {} at {}bps", port.as_ref().unwrap(), baudrate);
    match  MorpheusSerial::new(port.unwrap(), baudrate){
        Ok(mut serial) => {
            thread::sleep(Duration::from_millis(100));
            let joiner = serial.receive_frame();

            for _ in 0..20 {
              thread::sleep(Duration::from_millis(500));
              serial.send_frame(generated::Instructions::GetVersion {  }).unwrap();
            }

            joiner.join().unwrap().unwrap();
        }
        Err(error) => {
            eprintln!("Failed openning port: {}", error.to_string())
        }
    } 
}
