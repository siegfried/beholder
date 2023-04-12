use crate::result::{Error, Result};
use crate::schema::{binance_klines, binance_open_interest_summaries};
use binance_client::{
    api::Binance,
    futures::{
        market::FuturesMarket as FutureEndpoint,
        model::OpenInterestHist,
        websockets::{
            FuturesMarket, FuturesWebSockets as FutureWebSocket,
            FuturesWebsocketEvent as FutureWebSocketEvent,
        },
    },
    market::Market as SpotEndpoint,
    model::{KlineEvent, KlineSummaries, KlineSummary},
    websockets::{WebSockets as SpotWebSocket, WebsocketEvent as SpotWebSocketEvent},
};
use diesel::pg::{upsert::on_constraint, Pg, PgConnection};
use diesel::prelude::*;
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::Insertable;
use log::{info, warn};
use serde::Deserialize;
use std::sync::atomic::AtomicBool;
use std::{
    fs::OpenOptions,
    io::{BufReader, Write},
    path::Path,
    str::FromStr,
    vec::Vec,
};

#[derive(SqlType)]
#[postgres(type_name = "market")]
pub struct Market;

#[derive(Debug, PartialEq, AsExpression, Clone, Copy, clap::ArgEnum)]
#[sql_type = "Market"]
pub enum MarketEndpoint {
    Spot,
    USDM,
}

impl ToSql<Market, Pg> for MarketEndpoint {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            MarketEndpoint::Spot => out.write_all(b"SPOT")?,
            MarketEndpoint::USDM => out.write_all(b"USDM")?,
        }
        Ok(IsNull::No)
    }
}

impl MarketEndpoint {
    pub fn fetch(
        &self,
        query: &KlineQuery,
        interval: Option<String>,
        limit: Option<u16>,
        start_time: Option<u64>,
        end_time: Option<u64>,
        connection: &mut PgConnection,
    ) -> Result {
        let symbol = query.symbol.to_owned();
        let interval = interval.unwrap_or(query.interval.to_owned());
        let limit = limit.unwrap_or(query.limit);
        let KlineSummaries::AllKlineSummaries(summaries) = match self {
            MarketEndpoint::Spot => {
                info!("Downloading {}@{} from Binance Spot...", symbol, interval);
                let market: SpotEndpoint = Binance::new(None, None);
                market.get_klines(symbol, interval, limit, start_time, end_time)?
            }
            MarketEndpoint::USDM => {
                info!("Downloading {}@{} from Binance USDM...", symbol, interval);
                let market: FutureEndpoint = Binance::new(None, None);
                market.get_klines(symbol, interval, limit, start_time, end_time)?
            }
        };
        for summary in summaries {
            Kline::from_kline_summary(query.symbol.to_owned(), *self, summary)
                .upsert(connection)?;
        }
        Ok(())
    }

    pub fn watch(
        &self,
        queries: &[KlineQuery],
        interval: Option<String>,
        connection: &mut PgConnection,
    ) {
        let topics: Vec<String> = queries
            .into_iter()
            .map(|query| {
                let interval = interval.as_ref().unwrap_or(&query.interval);
                format!("{}@kline_{}", query.symbol.to_lowercase(), interval)
            })
            .collect();
        info!("Listen on topics: {:?}", topics);
        let keep_running = AtomicBool::new(true);

        match self {
            Self::Spot => {
                let mut web_socket: SpotWebSocket =
                    SpotWebSocket::new(|event: SpotWebSocketEvent| {
                        if let SpotWebSocketEvent::Kline(kline_event) = event {
                            if kline_event.kline.is_final_bar {
                                let kline: Kline = Kline::from_kline_event(*self, kline_event);
                                info!("Complete Kline received: {:?}", kline);
                                kline.upsert(connection).unwrap();
                            } else {
                                info!("Incomplete Kline received: {:?}", kline_event);
                            }
                        } else {
                            warn!("Unexpected Spot WS Event: {:?}", event);
                        };
                        Ok(())
                    });
                web_socket.connect_multiple_streams(&topics).unwrap();
                web_socket.event_loop(&keep_running).unwrap();
                web_socket.disconnect().unwrap();
            }

            Self::USDM => {
                let mut web_socket: FutureWebSocket =
                    FutureWebSocket::new(|event: FutureWebSocketEvent| {
                        if let FutureWebSocketEvent::Kline(kline_event) = event {
                            if kline_event.kline.is_final_bar {
                                let kline: Kline = Kline::from_kline_event(*self, kline_event);
                                info!("Complete Kline received: {:?}", kline);
                                kline.upsert(connection).unwrap();
                            } else {
                                info!("Incomplete Kline received: {:?}", kline_event);
                            }
                        } else {
                            warn!("Unexpected USDM WS Event: {:?}", event);
                        };
                        Ok(())
                    });
                web_socket
                    .connect_multiple_streams(&FuturesMarket::USDM, &topics)
                    .unwrap();
                web_socket.event_loop(&keep_running).unwrap();
                web_socket.disconnect().unwrap();
            }
        }
    }
}

impl FromStr for MarketEndpoint {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "spot" => Ok(Self::Spot),
            "usdm" => Ok(Self::USDM),
            _ => Err(Error::ParseStr(s.to_owned())),
        }
    }
}

#[derive(Debug, PartialEq, Insertable, AsChangeset)]
#[table_name = "binance_klines"]
pub struct Kline {
    source: MarketEndpoint,
    symbol: String,
    open_time: i64,
    close_time: i64,
    open: String,
    high: String,
    low: String,
    close: String,
    base_volume: String,
    quote_volume: String,
    buy_base_volume: String,
    buy_quote_volume: String,
    number_of_trades: i64,
}

impl Kline {
    pub fn from_kline_summary(symbol: String, source: MarketEndpoint, kline: KlineSummary) -> Self {
        Self {
            source: source,
            symbol: symbol,
            open_time: kline.open_time,
            close_time: kline.close_time,
            open: kline.open,
            high: kline.high,
            low: kline.low,
            close: kline.close,
            base_volume: kline.volume,
            quote_volume: kline.quote_asset_volume,
            buy_base_volume: kline.taker_buy_base_asset_volume,
            buy_quote_volume: kline.taker_buy_quote_asset_volume,
            number_of_trades: kline.number_of_trades,
        }
    }

    pub fn from_kline_event(source: MarketEndpoint, event: KlineEvent) -> Self {
        let kline = event.kline;

        Self {
            source: source,
            symbol: kline.symbol,
            open_time: kline.open_time,
            close_time: kline.close_time,
            open: kline.open,
            high: kline.high,
            low: kline.low,
            close: kline.close,
            base_volume: kline.volume,
            quote_volume: kline.quote_asset_volume,
            buy_base_volume: kline.taker_buy_base_asset_volume,
            buy_quote_volume: kline.taker_buy_quote_asset_volume,
            number_of_trades: kline.number_of_trades,
        }
    }

    pub fn upsert(&self, connection: &mut PgConnection) -> QueryResult<usize> {
        diesel::insert_into(binance_klines::table)
            .values(self)
            .on_conflict(on_constraint("binance_klines_pkey"))
            .do_update()
            .set(self)
            .execute(connection)
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct KlineQuery {
    pub symbol: String,
    pub interval: String,
    pub limit: u16,
}

impl KlineQuery {
    pub fn from_csv<P>(path: P) -> Result<Vec<Self>>
    where
        P: AsRef<Path>,
    {
        let file = OpenOptions::new().read(true).open(path)?;
        let mut reader = csv::Reader::from_reader(BufReader::new(file));
        Ok(reader.deserialize().collect::<csv::Result<Vec<Self>>>()?)
    }
}

#[derive(Debug, PartialEq, Insertable, AsChangeset)]
#[table_name = "binance_open_interest_summaries"]
pub struct OpenInterestSummary {
    symbol: String,
    interval: String,
    sum_open_interest: String,
    sum_open_interest_value: String,
    timestamp: i64,
}

impl OpenInterestSummary {
    fn from_open_interest_hist(interval: String, hist: OpenInterestHist) -> Result<Self> {
        Ok(Self {
            symbol: hist.symbol,
            interval,
            sum_open_interest: hist.sum_open_interest,
            sum_open_interest_value: hist.sum_open_interest_value,
            timestamp: hist.timestamp.try_into()?,
        })
    }

    fn upsert(&self, connection: &mut PgConnection) -> QueryResult<usize> {
        diesel::insert_into(binance_open_interest_summaries::table)
            .values(self)
            .on_conflict(on_constraint("binance_open_interest_summaries_pkey"))
            .do_update()
            .set(self)
            .execute(connection)
    }

    pub fn fetch(
        query: &KlineQuery,
        interval: Option<String>,
        limit: Option<u16>,
        start_time: Option<u64>,
        end_time: Option<u64>,
        connection: &mut PgConnection,
    ) -> Result {
        let market: FutureEndpoint = Binance::new(None, None);
        let symbol = &query.symbol;
        let interval = interval.unwrap_or(query.interval.to_owned());

        info!(
            "Downloading open interest summary of {}@{} ...",
            symbol, interval
        );

        let hists: Vec<OpenInterestHist> = market.open_interest_statistics(
            symbol.to_owned(),
            interval.to_owned(),
            limit,
            start_time,
            end_time,
        )?;

        for hist in hists {
            Self::from_open_interest_hist(interval.to_owned(), hist)?.upsert(connection)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Kline, KlineQuery, MarketEndpoint, OpenInterestSummary};
    use binance_client::futures::model::OpenInterestHist;
    use binance_client::model::{self, KlineEvent, KlineSummary};

    #[test]
    fn create_new_spot_kline_from_summary() {
        let summary = KlineSummary {
            open_time: 111,
            close_time: 222,
            open: "open".into(),
            high: "high".into(),
            low: "low".into(),
            close: "close".into(),
            volume: "base volume".into(),
            quote_asset_volume: "quote volume".into(),
            taker_buy_base_asset_volume: "buy base volume".into(),
            taker_buy_quote_asset_volume: "buy quote volume".into(),
            number_of_trades: 333,
        };

        let raw_kline = Kline {
            source: MarketEndpoint::Spot,
            symbol: "ETHBTC".into(),
            open_time: 111,
            close_time: 222,
            open: "open".into(),
            high: "high".into(),
            low: "low".into(),
            close: "close".into(),
            base_volume: "base volume".into(),
            quote_volume: "quote volume".into(),
            buy_base_volume: "buy base volume".into(),
            buy_quote_volume: "buy quote volume".into(),
            number_of_trades: 333,
        };

        assert_eq!(
            Kline::from_kline_summary("ETHBTC".into(), MarketEndpoint::Spot, summary),
            raw_kline
        )
    }

    #[test]
    fn create_new_spot_kline_from_kline_event() {
        let event = KlineEvent {
            symbol: "ETHBTC".into(),
            event_time: 111,
            event_type: "kline".into(),
            kline: model::Kline {
                first_trade_id: 1,
                last_trade_id: 2,
                interval: "1d".into(),
                symbol: "ETHBTC".into(),
                ignore_me: "ignore".into(),
                is_final_bar: true,
                open_time: 111,
                close_time: 222,
                open: "open".into(),
                high: "high".into(),
                low: "low".into(),
                close: "close".into(),
                volume: "base volume".into(),
                quote_asset_volume: "quote volume".into(),
                taker_buy_base_asset_volume: "buy base volume".into(),
                taker_buy_quote_asset_volume: "buy quote volume".into(),
                number_of_trades: 333,
            },
        };

        let raw_kline = Kline {
            source: MarketEndpoint::Spot,
            symbol: "ETHBTC".into(),
            open_time: 111,
            close_time: 222,
            open: "open".into(),
            high: "high".into(),
            low: "low".into(),
            close: "close".into(),
            base_volume: "base volume".into(),
            quote_volume: "quote volume".into(),
            buy_base_volume: "buy base volume".into(),
            buy_quote_volume: "buy quote volume".into(),
            number_of_trades: 333,
        };

        assert_eq!(
            Kline::from_kline_event(MarketEndpoint::Spot, event),
            raw_kline
        )
    }

    #[test]
    fn read_kline_argument_from_csv() {
        let results = KlineQuery::from_csv("tests/assets/kline_queries_1.csv").unwrap();

        let arguments = vec![
            KlineQuery {
                symbol: "BTCUSDT".into(),
                interval: "1d".into(),
                limit: 1500,
            },
            KlineQuery {
                symbol: "ETHUSDT".into(),
                interval: "1d".into(),
                limit: 1500,
            },
        ];

        assert_eq!(arguments, results)
    }

    #[test]
    fn create_open_interest_summary_from_hist() {
        let hist = OpenInterestHist {
            symbol: "BTCUSDT".into(),
            sum_open_interest: "20403.63700000".into(),
            sum_open_interest_value: "150570784.07809979".into(),
            timestamp: 1583127900000,
        };

        let summary = OpenInterestSummary {
            symbol: "BTCUSDT".into(),
            interval: "1d".into(),
            sum_open_interest: "20403.63700000".into(),
            sum_open_interest_value: "150570784.07809979".into(),
            timestamp: 1583127900000,
        };

        assert_eq!(
            summary,
            OpenInterestSummary::from_open_interest_hist("1d".into(), hist).unwrap()
        );
    }
}
