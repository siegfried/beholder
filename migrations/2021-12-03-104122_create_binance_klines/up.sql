CREATE TABLE binance_klines (
  symbol VARCHAR(30) NOT NULL,
  open_time BIGINT NOT NULL,
  close_time BIGINT NOT NULL,
  source market NOT NULL,
  open TEXT NOT NULL,
  high TEXT NOT NULL,
  low TEXT NOT NULL,
  close TEXT NOT NULL,
  base_volume TEXT NOT NULL,
  quote_volume TEXT NOT NULL,
  buy_base_volume TEXT NOT NULL,
  buy_quote_volume TEXT NOT NULL,
  number_of_trades BIGINT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
  PRIMARY KEY (symbol, open_time, close_time, source)
);

SELECT diesel_manage_updated_at('binance_klines');
