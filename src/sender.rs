use super::entry;
use reqwest::{Client, Error as HTTPError, Response, Result as HTTPResult};
use std::io;
use std::thread::sleep;
use std::time::Duration;

pub struct Sender {
    client: Client,
    url: String,
    token: String,
}

impl Sender {
    pub fn new(url: String, token: String) -> Sender {
        Sender {
            client: Client::new(),
            url: url,
            token: token,
        }
    }

    fn send(&self, series: &str, body: &str) -> HTTPResult<Response> {
        self.client
            .post(&format!("{}/{}", self.url, series))
            .body(serde_json::to_string(&body).unwrap())
            .bearer_auth(&self.token)
            .send()
    }

    fn is_not_retryable(error: &HTTPError) -> bool {
        !error.is_http() || !error.is_server_error()
    }

    pub fn send_with_retry_forever(&self, batch: &entry::Batch) -> io::Result<()> {
        for attempt in 0.. {
            if attempt > 0 {
                println!("[sender] Retry attempt {}", attempt);
            }

            let body = serde_json::to_string(&batch.entries)
                .map_err(|_| io::Error::new(io::ErrorKind::Other, "can not serialize batch"))?;

            let result = self
                .send(&batch.series, &body)
                .and_then(|response| response.error_for_status());

            match result {
                Ok(_) => return Ok(()),
                Err(cause) => {
                    if Sender::is_not_retryable(&cause) {
                        return Err(io::Error::new(
                            io::ErrorKind::Other,
                            format!("can not send a batch {:?}", cause),
                        ));
                    }
                }
            }

            sleep(Duration::from_secs(2));
        }
        return Ok(());
    }
}