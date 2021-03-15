use chrono::prelude::*;
use chrono::Duration;
use conv::*;
use structopt::StructOpt;
use yahoo_finance_api as yahoo;

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
    println!("{}", csv_header());
    for ticker in cli.tickers {
        if let Some(ti) = data_for(&provider, &ticker, quotes_from, quotes_to) {
            println!("{}", csv_line(&ti));
        } else {
            eprintln!("Could not get data for ticker {}", ticker);
        }
    }
}

struct TickerIndicators {
    symbol: String,
    currency: String,
    quote_times: Vec<DateTime<Utc>>,
    quote_values: Vec<f64>,
}

fn csv_header() -> &'static str {
    "period start,symbol,price,change %,min,max,30d avg"
}

fn csv_line(ti: &TickerIndicators) -> String {
    let thirty_day_avg = n_window_sma(30, &ti.quote_values)
        .and_then(|x| x.last().map(|v| format!("{}{:.2}", currency_symbol(&ti.currency), v)))
        .unwrap_or("-".to_owned());

    format!("{},{},{},{},{},{},{}",
            ti.quote_times.first().map(|x| x.to_rfc3339()).unwrap_or("-".to_owned()),
            ti.symbol,
            ti.quote_values.last().map(|x| format!("{}{:.2}", currency_symbol(&ti.currency), x)).unwrap_or("-".to_owned()),
            price_diff(&ti.quote_values).map(|x| format!("{:.2}%", x.1)).unwrap_or("-".to_owned()),
            min(&ti.quote_values).map(|x| format!("{}{:.2}", currency_symbol(&ti.currency), x)).unwrap_or("-".to_owned()),
            max(&ti.quote_values).map(|x| format!("{}{:.2}", currency_symbol(&ti.currency), x)).unwrap_or("-".to_owned()),
            thirty_day_avg)
}

fn currency_symbol(s: &str) -> &str {
    match s {
        "USD" => "$",
        _ => s,
    }
}

fn min(series: &[f64]) -> Option<f64> {
    Some(series.iter().cloned().fold(1./0. /* +inf */, f64::min))
}

fn max(series: &[f64]) -> Option<f64> {
    Some(series.iter().cloned().fold(-1./0. /* -inf */, f64::max))
}

fn price_diff(series: &[f64]) -> Option<(f64, f64)> {
    let first = series.first()?;
    let last = series.last()?;
    Some((last - first, (last - first)/first))
}

fn n_window_sma(n: usize, series: &[f64]) -> Option<Vec<f64>> {
    let w_size = f64::value_from(n).ok()?;
    let mut avgs = vec![];
    let mut running_total = 0.0;
    for (w_end, val) in series.iter().enumerate() {
        running_total += val;
        if w_end >= n {
            running_total -= series.get(w_end - n).unwrap();
        }

        // Has to be n-1 due to zero indexing.
        if w_end >= n - 1 {
            avgs.push(running_total / w_size);
        }
    }
    Some(avgs)
}

fn data_for(
    provider: &yahoo::YahooConnector,
    ticker: &str,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Option<TickerIndicators> {
    let interval = "1d";
    let quote = provider.get_quote_history_interval(ticker, from, to, interval);

    match quote {
        Ok(q) => {
            let res = q.chart.result.first()?; 
            let mut quote_times = vec![];
            let mut quote_values: Vec<f64> = vec![];
            for qt in q.quotes().unwrap() {
                let naive = NaiveDateTime::from_timestamp(qt.timestamp as i64, 0);
                quote_times.push(DateTime::<Utc>::from_utc(naive, Utc));
                quote_values.push(qt.adjclose);
            }
            Some(TickerIndicators {
                symbol: res.meta.symbol.clone(),
                currency: res.meta.currency.clone(),
                quote_times,
                quote_values,
            })
        }
        Err(e) => {
            println!("Error: {:?}", e);
            None
        }
    }
}
