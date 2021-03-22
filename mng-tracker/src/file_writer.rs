use xactor::*;
use std::fs::File;
use std::io::BufWriter;
use std::io::prelude::*;
use std::path::PathBuf;
use crate::messages::{PublishError, PublishTick, Flush};

pub struct FileWriter{
    buffer: BufWriter<File>,
    flush_seconds: u64,
}

pub fn new_file_writer(pb: &PathBuf, flush_seconds: u64) -> xactor::Result<FileWriter>  {
    let f = std::fs::File::create(pb)?;
    let buffer = BufWriter::new(f);
    Ok(FileWriter{buffer, flush_seconds})
}

#[async_trait::async_trait]
impl Actor for FileWriter {
    async fn started(&mut self, ctx: &mut Context<Self>) -> xactor::Result<()> {
        ctx.subscribe::<PublishTick>().await?;
        ctx.subscribe::<Flush>().await?;
        ctx.send_later(Flush{}, std::time::Duration::from_secs(self.flush_seconds));
        Ok(())
    }
}

#[async_trait::async_trait]
impl Handler<PublishTick> for FileWriter {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: PublishTick) {
        let to_write = format!("{}\n", msg.0).into_bytes();
        if let Err(e) = self.buffer.write(&to_write) {
            eprintln!("Filed to write tick to file:{}", e);
        }
    }
}

#[async_trait::async_trait]
impl Handler<Flush> for FileWriter {
    async fn handle(&mut self, ctx: &mut Context<Self>, _msg: Flush) {
        if let Err(_e) = self.buffer.flush() {
            if let Err(e) = Broker::from_registry()
                .await
                .and_then(|mut b| b.publish(PublishError("Unable to write to file.".to_owned())))
            {
                eprintln!("BrokerError:{}", e)
            }
        }
        ctx.send_later(Flush{}, std::time::Duration::from_secs(self.flush_seconds))
    }
}
