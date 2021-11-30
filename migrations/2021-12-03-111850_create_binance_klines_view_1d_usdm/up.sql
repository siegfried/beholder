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
