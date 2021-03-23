
use xactor::*;
use crate::messages::{PublishTick, LastN};
use std::collections::VecDeque;
use std::default::Default;

pub struct CacheActor {
    capacity: usize,
    cache: VecDeque<String>,
}

impl CacheActor {
    pub fn with_capacity(capacity: usize) -> Self {
        Self { capacity, ..Default::default()}
    }
}

impl Default for CacheActor {
    fn default() -> Self {
        Self{
            capacity: 50,
            cache: VecDeque::new(),
        }
    }
}

#[async_trait::async_trait]
impl Actor for CacheActor {
    async fn started(&mut self, ctx: &mut Context<Self>) -> xactor::Result<()> {
        self.cache = VecDeque::with_capacity(self.capacity);
        ctx.subscribe::<PublishTick>().await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl Handler<PublishTick> for CacheActor {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: PublishTick) {
        if self.cache.len() >= self.capacity {
            self.cache.pop_back();
        }
        self.cache.push_front(msg.0);
    }
}

#[async_trait::async_trait]
impl Handler<LastN> for CacheActor {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: LastN) -> Vec<String> {
        return self.cache.iter().take(msg.0).cloned().collect();
    }
}

