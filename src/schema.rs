table! {
    use diesel::sql_types::*;
    use crate::binance::Market;

    binance_klines (symbol, open_time, close_time, source) {
        symbol -> Varchar,
        open_time -> Int8,
        close_time -> Int8,
        source -> Market,
        open -> Text,
        high -> Text,
        low -> Text,
        close -> Text,
        base_volume -> Text,
        quote_volume -> Text,
        buy_base_volume -> Text,
        buy_quote_volume -> Text,
        number_of_trades -> Int8,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::binance::Market;

    binance_open_interest_summaries (symbol, interval, timestamp) {
        symbol -> Varchar,
        interval -> Varchar,
        timestamp -> Int8,
        sum_open_interest -> Text,
        sum_open_interest_value -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

allow_tables_to_appear_in_same_query!(
    binance_klines,
    binance_open_interest_summaries,
);
