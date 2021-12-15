CREATE TABLE binance_open_interest_summaries (
  symbol VARCHAR(30) NOT NULL,
  interval VARCHAR(10) NOT NULL,
  timestamp BIGINT NOT NULL,
  sum_open_interest TEXT NOT NULL,
  sum_open_interest_value TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
  PRIMARY KEY (symbol, interval, timestamp)
);

SELECT diesel_manage_updated_at('binance_open_interest_summaries');
