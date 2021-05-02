pub mod entry;
mod air_control;
mod sender;

use air_control::{AirControl, Response};
use sender::Sender;
use std::env;
use std::process::exit;
use std::thread;
use std::time::{Duration, UNIX_EPOCH, SystemTime};

const TICK: Duration = Duration::from_secs(1);
const FLUSH: Duration = Duration::from_secs(20);

fn epoch_now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() * 1000
}

fn collect_batches(control: &AirControl) -> Vec<entry::Batch> {
    let mut co2 = Vec::new();
    let mut t = Vec::new();

    let start = SystemTime::now();

    while SystemTime::now().duration_since(start).unwrap() < FLUSH {
        match control.read() {
            Ok(Response::CO2(value)) => {
                co2.push(entry::Entry {
                    ts: epoch_now(),
                    value: value as f64,
                });
            }
            Ok(Response::T(value)) => {
                t.push(entry::Entry {
                    ts: epoch_now(),
                    value: value as f64,
                });
            },
            Ok(_) => (),
            Err(error) => {
                println!("Can not take measurement: {:?}", error);
                ()
            }
        };
        thread::sleep(TICK);
    }

    vec![
        entry::Batch {
            series: "t".to_string(),
            entries: entry::Entries { entries: t },
        },
        entry::Batch {
            series: "co2".to_string(),
            entries: entry::Entries { entries: co2 },
        },
    ]
}

fn start(url: String, token: String) {
    println!();
    println!(
        "co2-monitor started. Measurements will be posted to the endpoint '{}'",
        url
    );
    println!();

    let sender = Sender::new(url, token);
    
    let control = AirControl::open().unwrap();

    loop {
        for batch in collect_batches(&control) {
            sender.send_with_retry_forever(&batch).unwrap();
        }
    }
}

fn main() {
    match (env::args().nth(1), env::args().nth(2)) {
        (Some(url), Some(token)) => start(url, token),
        _ => {
            println!("Usage {} [url] [token]", env::args().nth(0).unwrap());
            exit(1);
        }
    }
}