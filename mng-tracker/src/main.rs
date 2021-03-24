use async_std::task;
use chrono::prelude::*;
use chrono::Duration;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;
use text_io::read;
use tide::Request;
use xactor::*;

use mng_tracker::{
    new_file_writer, CacheActor, ErrorActor, LastN, PublishTick, PublisherActor, Ticker,
    TickerActor,
};

#[derive(StructOpt)]
struct Cli {
    /// The tickers to process
    tickers: Vec<String>,
    /// The period to use, expressed as 'yyyy-mm-dd'
    #[structopt(short = "p", long = "period", default_value = "")]
    period: String,
    /// The (optional) file to take a csv list of tickers to track from.
    #[structopt(parse(from_os_str), short = "f", long = "file")]
    from_file: Option<PathBuf>,
    /// Whether to write to stdout
    #[structopt(long = "stdout")]
    to_stdout: bool,
    /// The (optional) file to write output.
    #[structopt(parse(from_os_str), short = "o", long = "out")]
    out_file: Option<PathBuf>,
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

    let tickers = if let Some(f) = cli.from_file {
        let all_tickers = fs::read_to_string(f).expect("Could not read tickers from file");
        let all_tickers: Vec<String> = all_tickers.split(',').map(|x| x.to_owned()).collect();
        all_tickers
    } else {
        cli.tickers
    };

    match task::block_on(run_tickers(
        tickers,
        &quotes_from,
        &quotes_to,
        cli.to_stdout,
        &cli.out_file,
    )) {
        Ok(_) => eprintln!("Completed fine."),
        Err(e) => eprintln!("Error::{}", e),
    };
    Ok(())
}

async fn run_tickers(
    tickers: Vec<String>,
    quotes_from: &DateTime<Utc>,
    quotes_to: &DateTime<Utc>,
    to_stdout: bool,
    out_file: &Option<PathBuf>,
) -> xactor::Result<()> {
    let q_from = quotes_from.clone();
    let q_to = quotes_to.clone();

    // Start a publisher and an error handler.
    let _pa = if to_stdout {
        let pa = Supervisor::start(|| PublisherActor {}).await?;
        pa.send(PublishTick(Ticker::csv_header().to_owned()))?;
        Some(pa)
    } else {
        None
    };

    let _pe = Supervisor::start(|| ErrorActor("Error:".to_owned())).await?;

    let cache = Supervisor::start(|| CacheActor::with_capacity(100)).await?;

    let _fw = match out_file {
        Some(f) => {
            if let Ok(fw) = new_file_writer(&f, 30) {
                let fw_addr = fw.start().await?;
                fw_addr.send(PublishTick(Ticker::csv_header().to_owned()))?;
                Some(fw_addr)
            } else {
                eprintln!("Could not open file {:?}", f);
                None
            }
        }
        None => None,
    };


    eprintln!("Running");

    let port: usize = 8081;
    let conn = format!("127.0.0.1:{}", port);
    let state = State { cache };
    let mut app = tide::with_state(state);
    app.at("/tail/:count")
        .get(|req: Request<State>| async move {
            let n = req
                .param("count")
                .map(|x| x.parse::<usize>())
                .unwrap()
                .unwrap();
            let tail = req.state().cache.call(LastN(n)).await?;
            Ok(format!("{:?}", tail))
        });
    eprintln!("Listening on {}", conn);
    let tasks_future = start_tickers(tickers, q_from, q_to);
    let (t_err, app_err) = futures::join!(tasks_future, app.listen(conn));

    if let Err(e) = t_err {
        eprintln!("Tasks error:{}", e);
    }

    if let Err(e) = app_err {
        eprintln!("Web App Err:{}", e);
    }

    let _: String = read!("{}\n");
    eprintln!("All Done!");

    Ok(())
}

async fn start_tickers(
    tickers: Vec<String>,
    q_from: DateTime<Utc>,
    q_to: DateTime<Utc>,
) -> xactor::Result<Vec<Addr<TickerActor>>> {
    let mut tasks = vec![];
    for ticker in tickers.clone() {
        // Sleep before starting tickers (avoids too many simultaneous requests).
        task::sleep(std::time::Duration::from_millis(17)).await;
        // Start an actor, and send initial tick.
        let t = Supervisor::start(move || TickerActor {
            ticker: ticker.clone(),
            quotes_from: q_from,
            quotes_to: q_to,
        })
        .await?;
        tasks.push(t);
    }
    Ok(tasks)
}

#[derive(Clone)]
struct State {
    cache: xactor::Addr<CacheActor>,
}
