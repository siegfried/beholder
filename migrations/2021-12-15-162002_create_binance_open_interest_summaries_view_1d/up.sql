CREATE VIEW binance_open_interest_summaries_view_1d AS
  SELECT symbol,
         timestamp::DATE AS date,
         sum_open_interest,
         sum_open_interest_value
    FROM binance_open_interest_summaries_view
   WHERE interval = '1d';
