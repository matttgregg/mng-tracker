use chrono::prelude::*;
use chrono::Duration;
use structopt::StructOpt;
use yahoo_finance_api as yahoo;

use mng_tracker::ticker::Ticker;

#[derive(StructOpt)]
struct Cli {
    /// The tickers to process
    tickers: Vec<String>,
    #[structopt(short = "p", long = "period", default_value = "")]
    period: String,
}

fn main() {
    let cli = Cli::from_args();
    let period_start = NaiveDate::parse_from_str(&cli.period, "%Y-%m-%d");
    let quotes_from = match period_start {
        Ok(p) => DateTime::<Utc>::from_utc(p.and_hms(0, 0, 0), Utc),
        _ => {
            eprintln!(
                "No period given ({}) so defaulting to sixty days.",
                cli.period
            );
            Utc::now() + Duration::days(-60)
        },
    };

    let quotes_to = Utc::now();

    let provider = yahoo::YahooConnector::new();
    println!("{}", Ticker::csv_header());
    for ticker in cli.tickers {
        if let Some(ti) = Ticker::try_new(&provider, &ticker, quotes_from, quotes_to) {
            println!("{}", ti.csv_line());
        } else {
            eprintln!("Could not get data for ticker {}", ticker);
        }
    }
}

