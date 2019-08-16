use std::time::{Duration, SystemTime};
use std::thread::sleep;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::error::{Error as StdError};
use std::result::{Result as StdResult};
use measurement::{Measurement, CSV};
use reqwest::{Client, Result as HTTPResult, Error as HTTPError, Response};

pub struct Sender {
    client: Client,
    url: String,    
}

#[derive(Debug)]
pub struct Error {
    cause: HTTPError,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "Can not send measurements.")
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&self.cause)
    }
}

pub type Result = StdResult<(), Error>;

impl Sender {
    pub fn new(url: &str) -> Sender {
        Sender {
            client: Client::new(),
            url: url.to_string(),
        }
    }

    fn send(&self, measurements: &Vec<Measurement>) -> HTTPResult<Response> {
        self.client.post(&self.url)
            .body(measurements.as_csv())
            .send()
    }

    fn is_not_retryable(error: &HTTPError) -> bool {
        !error.is_http() || !error.is_server_error()
    }

    pub fn send_with_retry_forever(&self, measurements: &Vec<Measurement>) -> Result {
        for attempt in 0.. {
            if attempt > 0 {
                println!("[sender] Retry attempt {}", attempt);
            }

            let result = self.send(measurements).and_then(|response|
                response.error_for_status()
            );

            match result {
                Ok(_) => return Ok(()),
                Err(cause) => if Sender::is_not_retryable(&cause) {
                    return Err(Error{
                        cause: cause,
                    })
                }
            }

            sleep(Duration::from_secs(2));
        }
        return Ok(())
    }
}

//
// python mock-server.py
//
// #[test]
// fn test_send() {
//     let data = vec![
//         Measurement(SystemTime::now(), "t".to_string(), 12.12),
//         Measurement(SystemTime::now(), "co2".to_string(), 12.12),
//     ];
//     let sender = Sender::new("http://localhost:8000/measurements");
//     println!("{:?}", sender.send_with_retry_forever(&data));
// }