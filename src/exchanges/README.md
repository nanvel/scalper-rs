# Exchanges

Exchange responsibilities:

- provide basic symbol information (Symbol)
- keep shared state up-to-date
    - candles
    - order book
    - order flow
    - open interest (optional)
- submit orders (optional)
- provide order updates (optional)

## API

An exchange should implement the `Exchange` trait.

`start` method should return Symbol and SharedState, and then keep them updated. Use channels to log errors and
important
events.

`stop` method should gracefully stop all exchange activities and free resources.

`set_interval` should set `shared_candles_state.interval` and populate it with historical data.

`place_order` submits an order. This method should return immediately, spawn a new thread to submit the order,
then communicate updates or errors using channels.

`cancel_order` cancels an existing order. Similar to `place_order`, this method should return immediately.
