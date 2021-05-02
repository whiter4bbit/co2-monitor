use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Entry {
    pub ts: u64,
    pub value: f64,
}

#[derive(Debug, Serialize)]
pub struct Entries {
    pub entries: Vec<Entry>,
}

#[derive(Debug, Serialize)]
pub struct Batch {
    pub series: String,
    pub entries: Entries,
}