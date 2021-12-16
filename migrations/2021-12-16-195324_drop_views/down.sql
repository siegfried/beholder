BEGIN;

CREATE VIEW binance_klines_view AS
  SELECT symbol,
         source,
         TO_TIMESTAMP(open_time / 1000)::TIMESTAMP AS open_time,
         TO_TIMESTAMP((close_time + 1) / 1000) - TO_TIMESTAMP(open_time / 1000) AS interval,
         open::NUMERIC,
         high::NUMERIC,
         low::NUMERIC,
         close::NUMERIC,
         base_volume::NUMERIC,
         buy_base_volume::NUMERIC,
         quote_volume::NUMERIC,
         buy_quote_volume::NUMERIC,
         number_of_trades,
         created_at,
         updated_at
    FROM binance_klines;

CREATE VIEW binance_klines_view_1d AS
  SELECT symbol,
         source,
         open_time::DATE AS date,
         open,
         high,
         low,
         close,
         base_volume AS volume,
         COALESCE(buy_base_volume / NULLIF(base_volume, 0), 0) AS buy_percentage,
         number_of_trades
    FROM binance_klines_view
   WHERE interval = '1 day';

CREATE VIEW binance_klines_view_1d_spot AS
  SELECT symbol,
         date,
         open,
         high,
         low,
         close,
         volume,
         buy_percentage,
         number_of_trades
    FROM binance_klines_view_1d
   WHERE source = 'SPOT'
   ORDER BY symbol ASC,
            date ASC;

CREATE VIEW binance_klines_view_1d_usdm AS
  SELECT symbol,
         date,
         open,
         high,
         low,
         close,
         volume,
         buy_percentage,
         number_of_trades
    FROM binance_klines_view_1d
   WHERE source = 'USDM'
   ORDER BY symbol ASC,
            date ASC;

CREATE VIEW binance_klines_view_15m AS
  SELECT symbol,
         source,
         open_time,
         open,
         high,
         low,
         close,
         base_volume AS volume,
         COALESCE(buy_base_volume / NULLIF(base_volume, 0), 0) AS buy_percentage,
         number_of_trades
    FROM binance_klines_view
   WHERE interval = '15 minutes';

CREATE VIEW binance_open_interest_summaries_view AS
  SELECT symbol,
         interval,
         TO_TIMESTAMP(timestamp / 1000)::TIMESTAMP AS timestamp,
         sum_open_interest::NUMERIC,
         sum_open_interest_value::NUMERIC,
         created_at,
         updated_at
    FROM binance_open_interest_summaries;

CREATE VIEW binance_open_interest_summaries_view_1d AS
  SELECT symbol,
         timestamp::DATE AS date,
         sum_open_interest,
         sum_open_interest_value
    FROM binance_open_interest_summaries_view
   WHERE interval = '1d';

END TRANSACTION;
