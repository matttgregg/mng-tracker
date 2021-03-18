use chrono::prelude::*;
use yahoo_finance_api as yahoo;
use analyse::*;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Represent quote information for a ticker over a period of time.
pub struct Ticker {
    symbol: String,
    currency: String,
    quote_times: Vec<DateTime<Utc>>,
    quote_values: Vec<f64>,
}

impl Ticker {
    /// Attempt to construct a new ticker from a provider.
    ///
    /// Given a ticker symbol, and a connection, attempt to acquire quote data for a given time period.
    /// The quote data collected is the adjusted close price for each day.
    pub async fn try_new(
        provider: yahoo::YahooConnector,
        ticker: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Self> {
        let interval = "1d";
        let q = provider.get_quote_history_interval(ticker, from, to, interval).await?;

        let res = q.chart.result.first().ok_or("could not access results")?;
        let mut quote_times = vec![];
        let mut quote_values: Vec<f64> = vec![];
        for qt in q.quotes().unwrap() {
            let naive = NaiveDateTime::from_timestamp(qt.timestamp as i64, 0);
            quote_times.push(DateTime::<Utc>::from_utc(naive, Utc));
            quote_values.push(qt.adjclose);
        }
        Ok(Ticker {
            symbol: res.meta.symbol.clone(),
            currency: res.meta.currency.clone(),
            quote_times,
            quote_values,
        })
    }

    /// The header for csv output from this ticker.
    pub fn csv_header() -> &'static str {
        "period start,symbol,price,change %,min,max,30d avg"
    }

    /// The quote data in a single csv line.
    pub fn csv_line(&self) -> String {
        let thirty_day_avg = n_window_sma(30, &self.quote_values)
            .and_then(|x| {
                x.last()
                    .map(|v| format!("{}{:.2}", currency_symbol(&self.currency), v))
            })
            .unwrap_or("-".to_owned());

        format!(
            "{},{},{},{},{},{},{}",
            self.quote_times
                .first()
                .map(|x| x.to_rfc3339())
                .unwrap_or("-".to_owned()),
            self.symbol,
            self.quote_values
                .last()
                .map(|x| format!("{}{:.2}", currency_symbol(&self.currency), x))
                .unwrap_or("-".to_owned()),
            price_diff(&self.quote_values)
                .map(|x| format!("{:.2}%", x.1))
                .unwrap_or("-".to_owned()),
            min(&self.quote_values)
                .map(|x| format!("{}{:.2}", currency_symbol(&self.currency), x))
                .unwrap_or("-".to_owned()),
            max(&self.quote_values)
                .map(|x| format!("{}{:.2}", currency_symbol(&self.currency), x))
                .unwrap_or("-".to_owned()),
            thirty_day_avg
        )
    }
}

