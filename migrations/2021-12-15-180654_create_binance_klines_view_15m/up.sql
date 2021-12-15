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
