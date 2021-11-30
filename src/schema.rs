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
