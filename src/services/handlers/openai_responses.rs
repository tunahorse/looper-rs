use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{
        chat::ReasoningEffort,
        responses::{
            CreateResponse, CreateResponseArgs, FunctionCallOutput, FunctionCallOutputItemParam,
            FunctionToolCall, InputItem, InputParam, Item, OutputItem, Reasoning, ReasoningSummary,
            ResponseStreamEvent, Tool,
        },
    },
};

use async_recursion::async_recursion;
use async_trait::async_trait;

use anyhow::Result;
use futures::StreamExt;
use serde_json::Value;
use tokio::sync::{mpsc::Sender, oneshot};

use crate::{
    looper::AgentLoopState,
    services::{ChatHandler, handlers::openai_compatible::openai_compatible_client},
    types::{
        HandlerToLooperMessage, HandlerToLooperToolCallRequest, LooperToHandlerToolCallResult,
        LooperToolDefinition,
    },
};

pub struct OpenAIResponsesHandler {
    client: Client<OpenAIConfig>,
    previous_response_id: Option<String>,
    sender: Sender<HandlerToLooperMessage>,
    tools: Vec<Tool>,
    instructions: String,
    loop_state: AgentLoopState,
}

impl OpenAIResponsesHandler {
    pub fn new(sender: Sender<HandlerToLooperMessage>, system_message: &str) -> Result<Self> {
        let client = openai_compatible_client()?;

        Ok(OpenAIResponsesHandler {
            client,
            previous_response_id: None,
            sender,
            tools: Vec::new(),
            instructions: system_message.to_string(),
            loop_state: AgentLoopState::Continue,
        })
    }

    fn build_request(&self, input: Option<InputParam>) -> Result<CreateResponse> {
        let model = std::env::var("LOOPER_MODEL")
            .or_else(|_| std::env::var("ALCHEMY_MODEL"))
            .unwrap_or_else(|_| "gpt-5.2".to_string());

        let mut builder = CreateResponseArgs::default();
        match input {
            Some(i) => {
                builder
                    .model(model.clone())
                    .input(i)
                    .tools(self.tools.clone())
                    .reasoning(Reasoning {
                        effort: Some(ReasoningEffort::High),
                        summary: Some(ReasoningSummary::Concise),
                    })
                    .instructions(self.instructions.clone());
            }
            None => {
                builder
                    .model(model)
                    .tools(self.tools.clone())
                    .reasoning(Reasoning {
                        effort: Some(ReasoningEffort::High),
                        summary: Some(ReasoningSummary::Concise),
                    })
                    .instructions(self.instructions.clone());
            }
        }

        if let Some(ref prev_id) = self.previous_response_id {
            builder.previous_response_id(prev_id);
        }

        let request = builder.build()?;
        Ok(request)
    }

    async fn queue_function_call(
        &mut self,
        fc: FunctionToolCall,
        tool_call_receivers: &mut Vec<oneshot::Receiver<LooperToHandlerToolCallResult>>,
    ) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        let args = serde_json::from_str(&fc.arguments)?;

        self.handle_agent_loop_state(&fc.name, &args);

        let tcr = HandlerToLooperToolCallRequest {
            id: fc.call_id.clone(),
            name: fc.name.clone(),
            args,
            tool_result_channel: tx,
        };

        self.sender
            .send(HandlerToLooperMessage::ToolCallRequest(tcr))
            .await
            .unwrap();

        tool_call_receivers.push(rx);
        Ok(())
    }

    async fn stream_response(
        &mut self,
        input: Option<InputParam>,
    ) -> Result<(
        Vec<String>,
        Vec<oneshot::Receiver<LooperToHandlerToolCallResult>>,
        Option<String>,
    )> {
        let request = self.build_request(input)?;
        let mut stream = self.client.responses().create_stream(request).await?;

        let mut assistant_res_buf = Vec::new();
        let mut tool_call_receivers = Vec::new();
        let mut response_id: Option<String> = None;

        while let Some(event) = stream.next().await {
            match event {
                Ok(ResponseStreamEvent::ResponseOutputTextDelta(delta)) => {
                    let text = delta.delta.clone();
                    assistant_res_buf.push(text.clone());
                    self.sender
                        .send(HandlerToLooperMessage::Assistant(text))
                        .await
                        .unwrap();
                }
                Ok(ResponseStreamEvent::ResponseReasoningSummaryTextDelta(delta)) => {
                    let text = delta.delta.clone();
                    self.sender
                        .send(HandlerToLooperMessage::Thinking(text))
                        .await
                        .unwrap();
                }
                Ok(ResponseStreamEvent::ResponseReasoningSummaryTextDone(_)) => {
                    self.sender
                        .send(HandlerToLooperMessage::ThinkingComplete)
                        .await
                        .unwrap();
                }
                Ok(ResponseStreamEvent::ResponseOutputItemDone(item_done)) => {
                    if let OutputItem::FunctionCall(fc) = item_done.item {
                        self.queue_function_call(fc, &mut tool_call_receivers)
                            .await?;
                    }
                }
                Ok(ResponseStreamEvent::ResponseCompleted(completed)) => {
                    response_id = Some(completed.response.id.clone());
                }
                Ok(_) => {}
                Err(err) => {
                    println!("error: {err:?}");
                }
            }
        }

        Ok((assistant_res_buf, tool_call_receivers, response_id))
    }

    async fn collect_tool_results(
        tool_call_receivers: Vec<oneshot::Receiver<LooperToHandlerToolCallResult>>,
    ) -> Vec<(String, Value)> {
        futures::future::join_all(tool_call_receivers.into_iter().map(|rx| async move {
            let res = rx.await.unwrap();
            (res.id, res.value)
        }))
        .await
    }

    fn build_function_output_items(results: Vec<(String, Value)>) -> Vec<InputItem> {
        results
            .into_iter()
            .map(|(call_id, value)| {
                InputItem::Item(Item::FunctionCallOutput(FunctionCallOutputItemParam {
                    call_id,
                    output: FunctionCallOutput::Text(value.to_string()),
                    id: None,
                    status: None,
                }))
            })
            .collect()
    }

    #[async_recursion]
    async fn inner_send_message(&mut self, input: Option<InputParam>) -> Result<String> {
        let (assistant_res_buf, tool_call_receivers, response_id) =
            self.stream_response(input).await?;

        if let Some(id) = response_id {
            self.previous_response_id = Some(id);
        }

        let results = Self::collect_tool_results(tool_call_receivers).await;
        if !results.is_empty() {
            let input_items = Self::build_function_output_items(results);
            return self
                .inner_send_message(Some(InputParam::Items(input_items)))
                .await;
        }

        Ok(assistant_res_buf.join(""))
    }

    fn handle_agent_loop_state(&mut self, name: &str, args: &Value) {
        if name != "set_agent_loop_state" {
            return;
        }

        match args.get("state").and_then(Value::as_str) {
            Some("continue") => self.loop_state = AgentLoopState::Continue,
            Some("done") | None => {
                // If the model responds with a state that isn't supported
                // we should just end to avoid infinite loop
                self.loop_state = AgentLoopState::Done;
            }
            Some(_) => {
                self.loop_state = AgentLoopState::Done;
            }
        }
    }
}

#[async_trait]
impl ChatHandler for OpenAIResponsesHandler {
    async fn send_message(&mut self, message: &str) -> Result<()> {
        // Default to ending the turn unless the model explicitly requests continue.
        self.loop_state = AgentLoopState::Done;

        let input = InputParam::Text(message.to_string());
        self.inner_send_message(Some(input)).await?;

        while let AgentLoopState::Continue = &self.loop_state {
            self.loop_state = AgentLoopState::Done;
            self.inner_send_message(None).await?;
        }

        self.sender
            .send(HandlerToLooperMessage::TurnComplete)
            .await?;

        Ok(())
    }

    fn set_tools(&mut self, tools: Vec<LooperToolDefinition>) {
        self.tools = tools
            .into_iter()
            .map(|t| Tool::Function(t.into()))
            .collect();
    }
}
