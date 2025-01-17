extern crate binance as binance_client;
#[macro_use]
extern crate diesel;

mod binance;
mod result;
mod schema;

use crate::binance::{KlineQuery, MarketEndpoint};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use log::warn;
use result::Error;

fn main() {
    let cli = Cli::parse();
    cli.run();
}

#[derive(Parser)]
#[clap(author, version, about)]
#[clap(propagate_version = true, subcommand_required = true, arg_required_else_help = true)]
struct Cli {
    /// Silence all output
    #[clap(short, long, action)]
    quiet: bool,

    /// Increase message verbosity
    #[clap(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Timestamp (sec, ms, ns, none)
    #[clap(short, long, arg_enum, value_parser)]
    timestamp: Option<stderrlog::Timestamp>,

    #[clap(subcommand)]
    command: Commands,
}

impl Cli {
    fn run(self) {
        stderrlog::new()
            .module(module_path!())
            .quiet(self.quiet)
            .verbosity(self.verbose as usize)
            .timestamp(self.timestamp.unwrap_or(stderrlog::Timestamp::Off))
            .init()
            .unwrap();

        self.command.run();
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Snapshot data to database
    Snapshot {
        /// The database to store
        #[clap(short, long, value_parser)]
        database_url: String,

        /// Snapshot data to database
        #[clap(subcommand)]
        command: SnapshotCommands,
    },
}

impl Commands {
    fn run(self) {
        match self {
            Self::Snapshot {
                database_url,
                command,
            } => {
                let mut connection = PgConnection::establish(&database_url).unwrap();
                command.run(&mut connection);
            }
        }
    }
}

#[derive(Subcommand)]
enum SnapshotCommands {
    Binance {
        /// Binance Source
        #[clap(subcommand)]
        command: BinanceCommands,
    },
}

impl SnapshotCommands {
    fn run(self, connection: &mut PgConnection) {
        match self {
            Self::Binance { command } => command.run(connection),
        }
    }
}

#[derive(Subcommand)]
enum BinanceCommands {
    /// Fetch history Klines
    Kline {
        /// Choose a market
        #[clap(short, long, arg_enum, value_parser)]
        market: MarketEndpoint,

        /// The CSV file containing tasks of sync
        #[clap(short, long, value_parser)]
        csv: String,

        /// Use the interval instead of interval in CSV
        #[clap(short, long, value_parser)]
        interval: Option<String>,

        /// Use the limit instead of limits in CSV
        #[clap(short, long, value_parser)]
        limit: Option<u16>,

        /// Start time
        #[clap(long = "from", value_parser)]
        start_time: Option<DateTime<Utc>>,

        /// End time
        #[clap(long = "to", value_parser)]
        end_time: Option<DateTime<Utc>>,
    },

    /// Watch Klines in real time
    KlineStream {
        /// Choose a market
        #[clap(short, long, arg_enum, value_parser)]
        market: MarketEndpoint,

        /// The CSV file containing tasks of sync
        #[clap(short, long, value_parser)]
        csv: String,

        /// Use the interval instead of interval in CSV
        #[clap(short, long, value_parser)]
        interval: Option<String>,
    },

    /// Fetch open interest summaries
    OpenInterestSummary {
        /// The CSV file containing tasks of sync
        #[clap(short, long, value_parser)]
        csv: String,

        /// Use the interval instead of interval in CSV
        #[clap(short, long, value_parser)]
        interval: Option<String>,

        /// Use the limit instead of limits in CSV
        #[clap(short, long, value_parser)]
        limit: Option<u16>,

        /// Start time
        #[clap(long = "from", value_parser)]
        start_time: Option<DateTime<Utc>>,

        /// End time
        #[clap(long = "to", value_parser)]
        end_time: Option<DateTime<Utc>>,
    },
}

impl BinanceCommands {
    fn run(self, connection: &mut PgConnection) {
        match self {
            Self::Kline {
                market,
                csv,
                interval,
                limit,
                start_time,
                end_time,
            } => {
                let queries = KlineQuery::from_csv(csv).unwrap();

                for query in queries {
                    match market.fetch(
                        &query,
                        interval.to_owned(),
                        limit,
                        start_time.map(|t| t.timestamp_millis() as u64),
                        end_time.map(|t| t.timestamp_millis() as u64),
                        connection,
                    ) {
                        Ok(()) => (),
                        Err(Error::BinanceClient(error)) => {
                            warn!("Binance client failed: {}", error);
                            continue;
                        }
                        error => error.unwrap(),
                    }
                }
            }

            Self::KlineStream {
                market,
                csv,
                interval,
            } => {
                let queries = KlineQuery::from_csv(csv).unwrap();

                market.watch(&queries, interval, connection);
            }

            Self::OpenInterestSummary {
                csv,
                interval,
                start_time,
                end_time,
                limit,
            } => {
                let queries = KlineQuery::from_csv(csv).unwrap();

                for query in queries {
                    match binance::OpenInterestSummary::fetch(
                        &query,
                        interval.to_owned(),
                        limit,
                        start_time.map(|t| t.timestamp_millis() as u64),
                        end_time.map(|t| t.timestamp_millis() as u64),
                        connection,
                    ) {
                        Ok(()) => (),
                        Err(Error::BinanceClient(error)) => {
                            warn!("Binance client failed: {}", error);
                            continue;
                        }
                        error => error.unwrap(),
                    }
                }
            }
        }
    }
}
