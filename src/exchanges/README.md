# Exchanges

An exchange should implement the `src::exchanges::base::Exchange` trait.
Then list it in the `src::exchanges::factory` file to make it available for use.

Exchange responsibilities:

- provide basic symbol information (Symbol)
- keep shared state up-to-date
    - candles
    - order book
    - order flow
    - open interest (optional)
- submit orders (optional)
- provide order updates (optional)
