use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

pub fn sign(secret: &str, message: &str) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(message.as_bytes());
    let result = mac.finalize();
    hex::encode(result.into_bytes())
}

pub fn get_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as u64
}

pub fn build_signed_query(params: &[(&str, &str)], secret: &str) -> String {
    let mut query = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&");

    let signature = sign(secret, &query);
    query.push_str(&format!("&signature={}", signature));
    query
}

#[cfg(test)]
mod tests {
    use super::{build_signed_query, get_timestamp, sign};

    #[test]
    fn test_sign() {
        let secret = "test_secret";
        let message = "symbol=BTCUSDT&side=BUY";
        let signature = sign(secret, message);
        assert!(!signature.is_empty());
        assert_eq!(signature.len(), 64); // HMAC-SHA256 produces 64 hex chars
    }

    #[test]
    fn test_timestamp() {
        let ts = get_timestamp();
        assert!(ts > 0);
    }

    #[test]
    fn test_build_signed_query() {
        let params = vec![("symbol", "BTCUSDT"), ("side", "BUY")];
        let query = build_signed_query(&params, "secret");
        assert!(query.contains("symbol=BTCUSDT"));
        assert!(query.contains("side=BUY"));
        assert!(query.contains("signature="));
    }
}
