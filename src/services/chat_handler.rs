use crate::types::LooperToolDefinition;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait ChatHandler {
    async fn send_message(&mut self, message: &str) -> Result<()>;
    fn set_tools(&mut self, tools: Vec<LooperToolDefinition>);
}
