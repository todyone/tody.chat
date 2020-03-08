use anyhow::Error;
use async_trait::async_trait;
use meio::{Actor, Context};

pub struct Controller {}

#[async_trait]
impl Actor for Controller {
    type Message = ();

    async fn routine(&mut self, mut ctx: Context<Self>) -> Result<(), Error> {
        self.run(ctx).await?;
        Ok(())
    }
}

impl Controller {
    async fn run(&mut self, _: Context<Self>) -> Result<(), Error> {
        Ok(())
    }
}
