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
