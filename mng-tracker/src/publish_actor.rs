use xactor::*;
use crate::messages::PublishTick;

pub struct PublisherActor;

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
