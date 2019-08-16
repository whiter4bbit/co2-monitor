use std::time::{UNIX_EPOCH, SystemTime};

#[derive(Debug)]
pub struct Measurement(pub SystemTime, pub String, pub f32);

pub trait CSV {
    fn as_csv(&self) -> String;
}

impl CSV for Vec<Measurement> {
    fn as_csv(&self) -> String {
        let lines: Vec<String> = self.iter()
            .map(|m| {
                let Measurement(ts, key, value) = m;
                let epoch_ts = ts.duration_since(UNIX_EPOCH).unwrap().as_secs() * 1000;
                format!("{};{};{:.2}", epoch_ts, key, value)
            })
            .collect();
        lines.join("\n")
    }
}
