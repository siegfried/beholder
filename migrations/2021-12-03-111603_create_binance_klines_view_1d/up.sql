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
