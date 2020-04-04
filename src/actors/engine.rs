use anyhow::Error;
use async_trait::async_trait;
use meio::{wrapper, Actor, Address, Interaction, InteractionHandler};

wrapper!(Engine for EngineActor);

impl Engine {
    pub fn start() -> Self {
        let actor = EngineActor {};
        meio::spawn(actor)
    }
}

pub struct EngineActor {}

#[async_trait]
impl Actor for EngineActor {
    type Interface = Engine;

    fn generic_name() -> &'static str {
        "Engine"
    }

    async fn initialize(&mut self, _address: Address<Self>) -> Result<(), Error> {
        Ok(())
    }
}
