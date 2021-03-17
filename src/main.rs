use chrono::prelude::*;
use chrono::Duration;
use structopt::StructOpt;
use yahoo_finance_api as yahoo;
use async_std::{
    prelude::*, 
    task, 
};
use std::fs;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

use mng_tracker::ticker::Ticker;

#[derive(StructOpt)]
struct Cli {
    /// The tickers to process
    tickers: Vec<String>,
    #[structopt(short = "p", long = "period", default_value = "")]
    period: String,
    #[structopt(short = "f", long = "file", default_value = "")]
    from_file: String,
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
    let tickers = if cli.from_file != "" {
        let all_tickers = fs::read_to_string(cli.from_file).expect("Could not read tickers from file");
        let all_tickers: Vec<String> = all_tickers.split(',').map(|x| x.to_owned()).collect();
        all_tickers
    } else {
        cli.tickers
    };

    match task::block_on(run_tickers(tickers, &quotes_from, &quotes_to)) {
        Ok(_) => eprintln!("Completed fine."),
        Err(e) => eprintln!("Error::{}", e),
    };
}

fn spawn_and_log_error<F>(fut: F, tag: String) -> task::JoinHandle<()>
where
    F: Future<Output = Result<()>> + Send + 'static,
{
    task::spawn(async move {
        if let Err(e) = fut.await {
            eprintln!("{}:{}", tag, e)
        }
    })
}

async fn run_tickers(tickers: Vec<String>, quotes_from: &DateTime<Utc>, quotes_to: &DateTime<Utc>) -> Result<()> {
    let mut tasks = vec![];
    let q_from = quotes_from.clone();
    let q_to = quotes_to.clone();

    for ticker in tickers.clone() {
        let ticker_symbol = ticker.to_owned();
        let t = spawn_and_log_error(run_ticker(ticker_symbol, q_from, q_to), format!("{}", ticker));
        tasks.push((t, ticker));
    }

    // Wait for full completion.
    for (t, _) in tasks {
        t.await;
    }
    eprintln!("All Done!");

    Ok(())
}

async fn run_ticker(ticker: String, quotes_from: DateTime<Utc>, quotes_to: DateTime<Utc>) -> Result<()> {
    let provider = yahoo::YahooConnector::new();
    let ti = Ticker::try_new(provider, &ticker, quotes_from, quotes_to).await?;
    println!("{}", ti.csv_line());
    Ok(())
}

