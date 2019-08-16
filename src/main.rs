extern crate hidapi;
extern crate reqwest;

mod air_control;
mod measurement;
mod sender;

use air_control::{AirControl, Response};
use sender::Sender;
use std::time::{SystemTime, Duration};
use std::thread;
use std::env;
use std::process::exit;
use std::error::Error;
use measurement::{CSV, Measurement};

const TICK: Duration = Duration::from_secs(1);
const FLUSH: Duration = Duration::from_secs(20);

fn collect(control: &AirControl) -> Vec<Measurement> {
    let start = SystemTime::now();

    let mut measurements: Vec<Measurement> = Vec::new();
    while SystemTime::now().duration_since(start).unwrap() < FLUSH {
        let measurement: Option<Measurement> = match control.read() {
            Ok(Response::CO2(value)) => Some(Measurement(SystemTime::now(), "co2".to_string(), value)),
            Ok(Response::T(value)) => Some(Measurement(SystemTime::now(), "t".to_string(), value)),
            Ok(_) => None,
            Err(error) => {
                println!("Can not take measurement: {:?}", error);
                None
            }
        };
        measurement.map(|m| measurements.push(m));
        
        thread::sleep(TICK);
    }
    return measurements;
}

fn start(url: &str) {
    println!();
    println!("co2-monitor started. Measurements will be posted to the endpoint '{}'", url);
    println!();

    let sender = Sender::new(url);
    let control = AirControl::open().unwrap();

    loop {
        let measurements = collect(&control);
        sender.send_with_retry_forever(&measurements).map_err(
            |err| println!("Can not send the data: {}", err.source().unwrap())
        );
    }
}

fn main() {
    match env::args().nth(1) {
        Some(url) => start(&url),
        None => {
            println!("Usage {} [url]", env::args().nth(0).unwrap());
            exit(1);
        }
    }
}