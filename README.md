# Scalper

A desktop app for scalping (trading) written in Rust.

![ui 2025-12-25](https://github.com/nanvel/scalper-rs/releases/download/1.0.0/ui_20260103.png)

## Running the app

```shell
git pull git@github.com:nanvel/scalper-rs.git
cd scalper-rs
cargo run BTCUSDT
```

Command line options:

```text
Arguments:
  <SYMBOL>  

Options:
      --exchange <EXCHANGE>  
      --theme <THEME> (dark, light, auto)       
      --lot-size <LOT_SIZE> (lot size in quote)
      --sl-pnl <SL_PNL> (optional, flat position and cancel orders when PnL reaches this value)
```

Available exchanges (`src::exchanges::factory`):

- `binance_usd_futures` (default, trading is supported)
- `binance_spot`
- `binance_us_spot` (available for US IPs)
- `gateio_usd_futures`

Usage:

- `Esc` - exit the app
- `Shift + Up/Down` - scale in/out
- `Shift + Left/Right` - change interval
- `1`, `2`, `3`, `4` - choose lot multiplier
- `N` - reset aggressive volume and volume scale
- `+` - submit a market buy order (use lot size * multiplier)
- `-` - submit a marker sell order
- `0` (zero) - flat current position
- `C` - cancel all open orders
- `R` - reverse current position
- `Ctrl + LBC (Left Button Click)` - submit a limit order
- `Ctrl + Shift + LBC` - submit a stop order
- `Shift + LBC` - add a price alert (enable sound in config)

## Configuration

See `src/models/config.rs` for available configuration options.

The config file should be placed in `$HOME/.scalper-rs/config` (toml format).
Example:

```toml
theme = 'dark'
lot_size = 50
lot_mult_1 = 1
lot_mult_2 = 2
lot_mult_3 = 4
lot_mult_4 = 8
binance_access_key = 'Vb...'
binance_secret_key = '6V...'
sound = true
```

## ⚠️ Disclaimer

This software is provided for educational purposes only and is not financial advice.

**Trading involves substantial risk of loss.** You can lose some or all of your investment. The developers make no
guarantees about accuracy or reliability and are not liable for any losses or damages from using this software.

**Use at your own risk.** You are solely responsible for your trading decisions and ensuring compliance with applicable
laws and regulations.

By using this software, you accept all risks and responsibilities.
