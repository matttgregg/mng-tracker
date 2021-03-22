use xactor::*;
use crate::messages::PublishError;

pub struct ErrorActor(pub String);

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
        eprintln!("{}::{}", self.0, msg.0);
    }
}
