use yahoo_finance_api as yahoo;
use chrono::prelude::*;
use chrono::Duration;

fn main() {
    let provider = yahoo::YahooConnector::new();
    let ticker = "AAPL";
    let quote_from = Utc::now() + Duration::days(-10);
    let quote_to = Utc::now();

    let quote = provider.get_quote_history_interval(ticker, quote_from, quote_to, "1d");

    match quote {
        Ok(q) => {
            for res in q.chart.result.iter() {
                println!("Symbol: {}", res.meta.symbol);
                for qt in q.quotes().unwrap() {
                    println!("{:?} : {}", NaiveDateTime::from_timestamp(qt.timestamp as i64, 0), qt.adjclose);
                }
            }
        },
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}
