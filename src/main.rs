use chrono::prelude::*;
use chrono::Duration;
use structopt::StructOpt;
use yahoo_finance_api as yahoo;
use async_std::{
    prelude::*, 
    task, 
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

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

    println!("{}", Ticker::csv_header());
    task::block_on(run_tickers(&cli.tickers, &quotes_from, &quotes_to));
}

async fn run_tickers(tickers: &[String], quotes_from: &DateTime<Utc>, quotes_to: &DateTime<Utc>) -> Result<()> {
    let mut tasks = vec![];
    let q_from = quotes_from.clone();
    let q_to = quotes_to.clone();

    for ticker in tickers {
        let ticker_symbol = ticker.clone();
        let t = task::spawn(async move {
            run_ticker(&ticker_symbol, q_from, q_to).await;
        });
        tasks.push(t);
    }

    // Wait for full completion.
    for t in tasks {
        t.await;
    }

    Ok(())
}

async fn run_ticker(ticker: &str, quotes_from: DateTime<Utc>, quotes_to: DateTime<Utc>) -> Result<()> {
    let provider = yahoo::YahooConnector::new();
    let ti = Ticker::try_new(provider, ticker, quotes_from, quotes_to).await?;
    println!("{}", ti.csv_line());
    Ok(())
}

