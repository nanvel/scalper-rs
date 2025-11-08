use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Timestamp(u64);

impl Timestamp {
    pub fn now() -> Self {
        let now = SystemTime::now();
        let duration = now.duration_since(UNIX_EPOCH).expect("Time went backwards");

        Timestamp(duration.as_millis() as u64)
    }

    pub fn from_seconds(seconds: u64) -> Self {
        Timestamp(seconds * 1000)
    }

    pub fn from_milliseconds(mills: u64) -> Self {
        Timestamp(mills)
    }

    pub fn seconds(&self) -> u64 {
        self.0 / 1000
    }

    pub fn milliseconds(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}ms", self.0)
    }
}

impl From<u64> for Timestamp {
    fn from(value: u64) -> Self {
        Timestamp(value)
    }
}

impl From<Timestamp> for u64 {
    fn from(value: Timestamp) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::Timestamp;

    #[test]
    fn test_timestamp_display() {
        let ts = Timestamp::from(1625079600000);
        assert_eq!(format!("{}", ts), "1625079600000ms".to_string());
    }

    #[test]
    fn test_timestamp_from_u64() {
        let ts: Timestamp = 1625079600000u64.into();
        assert_eq!(ts.0, 1625079600000);
    }

    #[test]
    fn test_timestamp_to_u64() {
        let ts = Timestamp::from(1625079600000);
        let value: u64 = ts.into();
        assert_eq!(value, 1625079600000);
    }
}
