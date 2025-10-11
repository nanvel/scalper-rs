mod data;
mod streams;
use streams::start_candles_stream;

#[tokio::main]
async fn main() {
    let candles_store = start_candles_stream("BTCUSDT".to_string(), "5m".to_string(), 200).await;
    let candles_store_clone = candles_store.clone();

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        println!("2s");

        let buffer = candles_store_clone.read().await;

        println!("{:#?}", buffer.to_vec())
    }
}
