CREATE VIEW binance_open_interest_summaries_view AS
  SELECT symbol,
         period,
         TO_TIMESTAMP(timestamp / 1000)::TIMESTAMP AS timestamp,
         sum_open_interest::NUMERIC,
         sum_open_interest_value::NUMERIC,
         created_at,
         updated_at
    FROM binance_open_interest_summaries;
