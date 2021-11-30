extern crate binance as binance_client;
#[macro_use]
extern crate diesel;

mod binance;
mod result;
mod schema;

use crate::binance::{KlineQuery, MarketEndpoint};
use clap::{crate_authors, crate_description, crate_name, crate_version, App, AppSettings, Arg};
use diesel::pg::PgConnection;
use diesel::prelude::*;

fn main() {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .about("Silence all output"),
        )
        .arg(
            Arg::new("verbosity")
                .short('v')
                .multiple_occurrences(true)
                .about("Increase message verbosity"),
        )
        .arg(
            Arg::new("timestamp")
                .short('t')
                .about("Prepend log lines with a timestamp")
                .takes_value(true)
                .possible_values(&["none", "sec", "ms", "ns"]),
        )
        .subcommand(
            App::new("snapshot")
                .about("Snapshot data to database")
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .arg(
                    Arg::new("database-url")
                        .short('d')
                        .long("database-url")
                        .value_name("URL")
                        .about("The database to store.")
                        .takes_value(true)
                        .required(true),
                )
                .subcommand(
                    App::new("binance")
                        .about("Binance Source")
                        .setting(AppSettings::SubcommandRequiredElseHelp)
                        .subcommand(
                            App::new("kline")
                                .about("Kline Data")
                                .setting(AppSettings::ArgRequiredElseHelp)
                                .arg(
                                    Arg::new("market")
                                        .long("market")
                                        .value_name("MARKET")
                                        .about("Choose a market")
                                        .takes_value(true)
                                        .possible_values(&["spot", "usdm"])
                                        .required(true),
                                )
                                .arg(
                                    Arg::new("csv")
                                        .long("csv")
                                        .value_name("FILE")
                                        .about("The CSV file containing tasks of sync")
                                        .takes_value(true)
                                        .required(true),
                                )
                                .arg(
                                    Arg::new("limit")
                                        .long("limit")
                                        .value_name("NUMBER")
                                        .about("Use the limit instead of limits in CSV")
                                        .takes_value(true),
                                ),
                        ),
                ),
        )
        .get_matches();

    let verbose: usize = matches.occurrences_of("verbosity").try_into().unwrap();
    let quiet = matches.is_present("quiet");
    let ts: stderrlog::Timestamp = matches
        .value_of("timestamp")
        .map(|v| v.parse().unwrap())
        .unwrap_or(stderrlog::Timestamp::Off);

    stderrlog::new()
        .module(module_path!())
        .quiet(quiet)
        .verbosity(verbose)
        .timestamp(ts)
        .init()
        .unwrap();

    if let Some(ref matches) = matches.subcommand_matches("snapshot") {
        let connection = {
            let database_url = matches.value_of("database-url").unwrap();
            PgConnection::establish(&database_url)
                .expect(&format!("Error connecting to {}", database_url))
        };

        if let Some(ref binance_matches) = matches.subcommand_matches("binance") {
            if let Some(ref kline_matches) = binance_matches.subcommand_matches("kline") {
                let queries = {
                    let path = kline_matches.value_of("csv").unwrap();
                    KlineQuery::from_csv(path).unwrap()
                };
                run_snapshot_binance_kline(
                    kline_matches.value_of_t_or_exit("market"),
                    &queries,
                    kline_matches
                        .value_of("limit")
                        .map(|limit| limit.parse().unwrap()),
                    &connection,
                );
            }
        }
    }
}

fn run_snapshot_binance_kline(
    market: MarketEndpoint,
    queries: &[KlineQuery],
    limit: Option<u16>,
    connection: &PgConnection,
) {
    for query in queries {
        market.fetch(&query, limit, &connection).unwrap();
    }
}
