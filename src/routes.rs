use std::convert::Infallible;
use warp::Reply;
use warp::{self, Filter};
use crate::generated;
use crate::MorpheusSerial;
use tokio;

async fn send_get_instruction(serial: MorpheusSerial, inst: generated::Instructions) -> Result<impl warp::reply::Reply, Infallible> {
    let mut rx = serial.rx_queue.resubscribe();
    
    serial.send_frame(inst).await.unwrap();
    
    let res : Box<dyn Reply> = tokio::select!(
        fb = rx.recv() => {
            let fb = fb.unwrap();
            Box::new(warp::reply::json(&fb))
        }
        _ = tokio::time::sleep(tokio::time::Duration::from_millis(500)) => {Box::new(warp::reply::reply())},
    );

    Ok(res)
}

fn with_serial(serial: &MorpheusSerial) -> impl Filter<Extract = (MorpheusSerial,), Error = Infallible> + Clone {
    let ser = serial.clone();
    warp::any().map( move || ser.clone())
} 

fn with_instruction(inst: generated::Instructions) -> impl Filter<Extract = (generated::Instructions,), Error = Infallible> + Clone {
    warp::any().map(move || {inst.clone()})
} 

/// GET /version
pub fn morpheus_version(serial: &MorpheusSerial) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection>  + Clone{
    warp::path("version")
        .and(warp::get())
        .and(with_serial(&serial))
        .and(with_instruction(generated::Instructions::GetVersion {  }))
        .and_then(send_get_instruction)
}

/// GET /sleep_pin
pub fn morpheus_sleep_pin(serial: &MorpheusSerial) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection>  + Clone{
    warp::path!("sleep_pin" / u16)
        .and(warp::get())
        .and(with_serial(&serial))
        .and_then(move |a, ser| {
            let inst = generated::Instructions::SleepPin { pre_sleep_time: a };
            send_get_instruction(ser, inst)})
}

pub fn morpheus_routes(serial: &MorpheusSerial) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    morpheus_version(&serial.clone())
    .or(morpheus_sleep_pin(&serial))
}