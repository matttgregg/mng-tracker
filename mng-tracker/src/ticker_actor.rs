use chrono::prelude::*;
use xactor::*;
use yahoo_finance_api as yahoo;
use crate::messages::{Tick, PublishTick, PublishError};
use crate::Ticker;

pub struct TickerActor{
    pub ticker: String,
    pub quotes_from: DateTime<Utc>,
    pub quotes_to: DateTime<Utc>,
}

#[async_trait::async_trait]
impl Actor for TickerActor {
    async fn started(&mut self, ctx: &mut Context<Self>) -> xactor::Result<()> {
        ctx.send_later(Tick{}, std::time::Duration::from_secs(0));
        Ok(())
    }
}

#[async_trait::async_trait]
impl Handler<Tick> for TickerActor {
    async fn handle(&mut self, ctx: &mut Context<Self>, _msg: Tick) {
        match run_ticker(self.ticker.clone(), self.quotes_from, self.quotes_to).await {
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
                    b.publish(PublishError(format!("{}|{:?}", self.ticker, e)))
                }) {
                    eprintln!("BrokerError:{}", e)
                }
            }
        }

        ctx.send_later(Tick{}, std::time::Duration::from_secs(30))
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
