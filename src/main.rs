use async_std::task;
use chrono::prelude::*;
use chrono::Duration;
use std::fs;
use structopt::StructOpt;
use xactor::*;
use yahoo_finance_api as yahoo;

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

#[xactor::main]
async fn main() -> Result<()> {
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
        }
    };

    let quotes_to = Utc::now();

    let tickers = if cli.from_file != "" {
        let all_tickers =
            fs::read_to_string(cli.from_file).expect("Could not read tickers from file");
        let all_tickers: Vec<String> = all_tickers.split(',').map(|x| x.to_owned()).collect();
        all_tickers
    } else {
        cli.tickers
    };

    match task::block_on(run_tickers(tickers, &quotes_from, &quotes_to)) {
        Ok(_) => eprintln!("Completed fine."),
        Err(e) => eprintln!("Error::{}", e),
    };
    Ok(())
}

async fn run_tickers(
    tickers: Vec<String>,
    quotes_from: &DateTime<Utc>,
    quotes_to: &DateTime<Utc>,
) -> xactor::Result<()> {
    let mut tasks = vec![];
    let q_from = quotes_from.clone();
    let q_to = quotes_to.clone();

    // Start a publisher and an error handler.
    let pa = PublisherActor.start().await?;
    let pe = ErrorActor.start().await?;

    pa.send(PublishTick(Ticker::csv_header().to_owned()))?;
    pe.send(PublishError("--".to_owned()))?;

    for ticker in tickers.clone() {
        let ticker_symbol = ticker.to_owned();
        task::sleep(std::time::Duration::from_millis(37)).await;
        // Start an actor, and send initial tick.
        let t = TickerActor.start().await?;
        if let Err(e) = t.send(Tick {
            ticker: ticker_symbol,
            quotes_from: q_from,
            quotes_to: q_to,
        }) {
            eprintln!("Failed to start ticker: {} {}", ticker, e)
        }
        tasks.push(t);
    }

    // Wait for full completion.
    for t in tasks {
        t.wait_for_stop().await;
    }
    eprintln!("All Done!");

    Ok(())
}

#[message]
#[derive(Clone)]
struct PublishTick(String);

#[message]
#[derive(Clone)]
struct PublishError(String);

#[message]
#[derive(Clone)]
struct Tick {
    ticker: String,
    quotes_from: DateTime<Utc>,
    quotes_to: DateTime<Utc>,
}

struct PublisherActor;

#[async_trait::async_trait]
impl Actor for PublisherActor {
    async fn started(&mut self, ctx: &mut Context<Self>) -> xactor::Result<()> {
        ctx.subscribe::<PublishTick>().await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl Handler<PublishTick> for PublisherActor {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: PublishTick) {
        println!("{}", msg.0);
    }
}

struct ErrorActor;

#[async_trait::async_trait]
impl Actor for ErrorActor {
    async fn started(&mut self, ctx: &mut Context<Self>) -> xactor::Result<()> {
        ctx.subscribe::<PublishError>().await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl Handler<PublishError> for ErrorActor {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: PublishError) {
        eprintln!("ERR::{}", msg.0);
    }
}

struct TickerActor;

#[async_trait::async_trait]
impl Actor for TickerActor {}

#[async_trait::async_trait]
impl Handler<Tick> for TickerActor {
    async fn handle(&mut self, ctx: &mut Context<Self>, msg: Tick) {
        let next_tick = Tick {
            ticker: msg.ticker.clone(),
            quotes_from: msg.quotes_from.clone(),
            quotes_to: msg.quotes_to.clone(),
        };
        match run_ticker(msg.ticker, msg.quotes_from, msg.quotes_to).await {
            Ok(p) => {
                if let Err(e) = Broker::from_registry()
                    .await
                    .and_then(|mut b| b.publish(PublishTick(p)))
                {
                    eprintln!("BrokerError:{}", e)
                }
            }
            Err(e) => {
                if let Err(e) = Broker::from_registry().await.and_then(|mut b| {
                    b.publish(PublishError(format!("{}|{:?}", next_tick.ticker, e)))
                }) {
                    eprintln!("BrokerError:{}", e)
                }
            }
        }

        ctx.send_later(next_tick, std::time::Duration::from_secs(30))
    }
}

async fn run_ticker(
    ticker: String,
    quotes_from: DateTime<Utc>,
    quotes_to: DateTime<Utc>,
) -> Result<String> {
    let provider = yahoo::YahooConnector::new();
    let ti = Ticker::try_new(provider, &ticker, quotes_from, quotes_to).await?;
    Ok(ti.csv_line())
}
