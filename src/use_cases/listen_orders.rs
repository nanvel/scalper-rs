use crate::binance::BinanceClient;
use crate::binance::start_account_stream;
use crate::models::Config;
use std::thread;
use tokio::runtime;

pub fn listen_orders(config: &Config, symbol: String) {
    let client = BinanceClient::new(
        config.binance_access_key.clone(),
        config.binance_secret_key.clone(),
    );
    thread::spawn(move || {
        let rt = runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .expect("Failed to build tokio runtime for open interest");

        rt.block_on(async move {
            loop {
                let listen_key = client.get_listen_key().await.unwrap();
                start_account_stream(listen_key, symbol.clone())
                    .await
                    .unwrap();
            }
        });
    });
}
