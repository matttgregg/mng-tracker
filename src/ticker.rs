use chrono::prelude::*;
use conv::*;
use yahoo_finance_api as yahoo;

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

/// Acquire a currency symbol for the long form currency symbol.
///
/// The only currently handled case is to represent USD as $. In all other
/// cases the original long form symbol is returned.
///
/// ```
/// # use mng_tracker::ticker::currency_symbol;
/// assert_eq!(currency_symbol("USD"), "$");
/// assert_eq!(currency_symbol("GBP"), "GBP");
/// ````
pub fn currency_symbol(s: &str) -> &str {
    match s {
        "USD" => "$",
        _ => s,
    }
}

/// Find the minimum of a sequence of floats.
///
/// ```
/// # use mng_tracker::ticker::min;
/// let result = min(&vec![11.2, -13.6, 0.004, 500.9, -27.1, -26.2, 5.4]).expect("should be a float");
/// assert_eq!(result, -27.1);
/// ```
pub fn min(series: &[f64]) -> Option<f64> {
    Some(series.iter().cloned().fold(1. / 0. /* +inf */, f64::min))
}

/// Find the maximum of a sequence of floats.
///
/// ```
/// # use mng_tracker::ticker::max;
/// let result = max(&vec![11.2, -13.6, 0.004, 500.9, -27.1, -26.2, 5.4]).expect("should be a float");
/// assert_eq!(result, 500.9);
/// ```
pub fn max(series: &[f64]) -> Option<f64> {
    Some(series.iter().cloned().fold(-1. / 0. /* -inf */, f64::max))
}

/// Find the price difference from the first to the last in a sequence of floats, as absolute and percentage.
///
/// The returned value is a tuple of the absolute difference, and the difference as a percentage of the initial value.
/// ```
/// # use mng_tracker::ticker::price_diff;
/// let result = price_diff(&vec![1.0, 11.2, -13.6, 0.004, 500.9, -27.1, -26.2, 5.4, 2.0]).expect("should be a tuple");
/// assert_eq!(result, (1.0, 100.0));
/// ```
pub fn price_diff(series: &[f64]) -> Option<(f64, f64)> {
    let first = series.first()?;
    let last = series.last()?;
    Some((last - first, 100.0 * (last - first) / first))
}

/// Produce a simple moving average a the given sequence of floats, and given window size.
///
/// ```
/// # use mng_tracker::ticker::n_window_sma;
/// let data = vec![1.0, 11.2, -13.6, 0.004, 500.9, -27.1, -26.2, 5.4, 2.0];
/// let averages = n_window_sma(2, &data).expect("should be able to compute averages");
/// assert_eq!(averages.len(), data.len() - 1);
/// assert!((averages[0] - (&data[0] + &data[1]) / 2.0).abs() < 0.001);
/// assert!((averages[1] - (&data[1] + &data[2]) / 2.0).abs() < 0.001);
/// assert!((averages[7] - (&data[7] + &data[8]) / 2.0).abs() < 0.001);
/// ```
pub fn n_window_sma(n: usize, series: &[f64]) -> Option<Vec<f64>> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_min() {
        assert_eq!(min(&vec![-1.5, 0.0, -45.3, 27.0, 0.7]).unwrap(), -45.3);
    }

    #[test]
    fn test_max() {
        assert_eq!(max(&vec![-1.5, 0.0, -45.3, 27.0, 0.7]).unwrap(), 27.0);
    }
}

