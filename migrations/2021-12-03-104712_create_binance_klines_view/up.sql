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
