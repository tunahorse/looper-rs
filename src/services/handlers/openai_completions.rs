use async_openai::{
    Client,
    config::OpenAIConfig,
    types::chat::{
        ChatChoiceStream, ChatCompletionMessageToolCall, ChatCompletionMessageToolCallChunk,
        ChatCompletionMessageToolCalls, ChatCompletionRequestAssistantMessage,
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestToolMessage,
        ChatCompletionRequestUserMessageArgs, ChatCompletionTools, CreateChatCompletionRequest,
        CreateChatCompletionRequestArgs, FinishReason, ReasoningEffort,
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

pub struct OpenAIChatHandler {
    client: Client<OpenAIConfig>,
    messages: Vec<ChatCompletionRequestMessage>,
    sender: Sender<HandlerToLooperMessage>,
    tools: Vec<ChatCompletionTools>,
    loop_state: AgentLoopState,
}

impl OpenAIChatHandler {
    pub fn new(sender: Sender<HandlerToLooperMessage>, system_message: &str) -> Result<Self> {
        let client = openai_compatible_client()?;
        let system_message = ChatCompletionRequestSystemMessageArgs::default()
            .content(system_message)
            .build()?
            .into();

        let messages = vec![system_message];
        let tools = Vec::new();

        Ok(OpenAIChatHandler {
            client,
            messages,
            sender,
            tools,
            loop_state: AgentLoopState::Continue,
        })
    }

    fn build_request(&self) -> Result<CreateChatCompletionRequest> {
        let model = std::env::var("LOOPER_MODEL")
            .or_else(|_| std::env::var("ALCHEMY_MODEL"))
            .unwrap_or_else(|_| "gpt-5.2".to_string());

        let request = CreateChatCompletionRequestArgs::default()
            .model(model)
            .max_completion_tokens(50000u32)
            .messages(self.messages.clone())
            .tools(self.tools.clone())
            .reasoning_effort(ReasoningEffort::Low)
            .build()?;

        Ok(request)
    }

    fn apply_tool_call_chunks(
        tool_calls: &mut Vec<ChatCompletionMessageToolCall>,
        chunks: Vec<ChatCompletionMessageToolCallChunk>,
    ) {
        for chunk in chunks {
            let index = chunk.index as usize;

            while tool_calls.len() <= index {
                tool_calls.push(ChatCompletionMessageToolCall {
                    id: String::new(),
                    function: Default::default(),
                });
            }

            let tool_call = &mut tool_calls[index];
            if let Some(id) = chunk.id {
                tool_call.id = id;
            }
            if let Some(function_chunk) = chunk.function {
                if let Some(name) = function_chunk.name {
                    tool_call.function.name = name;
                }
                if let Some(arguments) = function_chunk.arguments {
                    tool_call.function.arguments.push_str(&arguments);
                }
            }
        }
    }

    async fn queue_tool_calls(
        &mut self,
        tool_calls: &[ChatCompletionMessageToolCall],
        tool_call_receivers: &mut Vec<oneshot::Receiver<LooperToHandlerToolCallResult>>,
    ) -> Result<()> {
        for tool_call in tool_calls {
            let id = tool_call.id.clone();
            let name = tool_call.function.name.clone();
            let args: Value = serde_json::from_str(&tool_call.function.arguments)?;

            self.handle_agent_loop_state(&name, &args);
            let (tx, rx) = oneshot::channel();

            let tcr = HandlerToLooperToolCallRequest {
                id,
                name,
                args,
                tool_result_channel: tx,
            };

            self.sender
                .send(HandlerToLooperMessage::ToolCallRequest(tcr))
                .await
                .unwrap();

            tool_call_receivers.push(rx);
        }

        Ok(())
    }

    async fn handle_stream_choice(
        &mut self,
        choice: ChatChoiceStream,
        assistant_res_buf: &mut Vec<String>,
        tool_calls: &mut Vec<ChatCompletionMessageToolCall>,
        tool_call_receivers: &mut Vec<oneshot::Receiver<LooperToHandlerToolCallResult>>,
    ) -> Result<()> {
        if let Some(content) = choice.delta.content {
            assistant_res_buf.push(content.clone());
            self.sender
                .send(HandlerToLooperMessage::Assistant(content))
                .await
                .unwrap();
        }

        if let Some(tool_call_chunks) = choice.delta.tool_calls {
            Self::apply_tool_call_chunks(tool_calls, tool_call_chunks);
        }

        if matches!(choice.finish_reason, Some(FinishReason::ToolCalls)) {
            self.queue_tool_calls(tool_calls, tool_call_receivers)
                .await?;
        }

        Ok(())
    }

    async fn stream_response(
        &mut self,
    ) -> Result<(
        Vec<String>,
        Vec<ChatCompletionMessageToolCall>,
        Vec<oneshot::Receiver<LooperToHandlerToolCallResult>>,
    )> {
        let request = self.build_request()?;
        let mut stream = self.client.chat().create_stream(request).await?;
        let mut assistant_res_buf = Vec::new();
        let mut tool_calls = Vec::new();
        let mut tool_call_receivers = Vec::new();

        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    for choice in response.choices {
                        self.handle_stream_choice(
                            choice,
                            &mut assistant_res_buf,
                            &mut tool_calls,
                            &mut tool_call_receivers,
                        )
                        .await?;
                    }
                }
                Err(err) => {
                    println!("error: {err:?}");
                }
            }
        }

        Ok((assistant_res_buf, tool_calls, tool_call_receivers))
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

    fn append_tool_messages(
        &mut self,
        tool_calls: Vec<ChatCompletionMessageToolCall>,
        tool_results: Vec<(String, Value)>,
    ) {
        let assistant_tool_calls: Vec<ChatCompletionMessageToolCalls> =
            tool_calls.into_iter().map(Into::into).collect();

        self.messages.push(
            ChatCompletionRequestAssistantMessage {
                content: None,
                tool_calls: Some(assistant_tool_calls),
                ..Default::default()
            }
            .into(),
        );

        for (tool_call_id, response) in tool_results {
            self.messages.push(
                ChatCompletionRequestToolMessage {
                    content: response.to_string().into(),
                    tool_call_id,
                }
                .into(),
            );
        }
    }

    #[async_recursion]
    async fn inner_send_message(&mut self) -> Result<String> {
        let (assistant_res_buf, tool_calls, tool_call_receivers) = self.stream_response().await?;
        let tool_results = Self::collect_tool_results(tool_call_receivers).await;

        if tool_results.is_empty() {
            return Ok(assistant_res_buf.join(""));
        }

        self.append_tool_messages(tool_calls, tool_results);
        self.inner_send_message().await
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
impl ChatHandler for OpenAIChatHandler {
    async fn send_message(&mut self, message: &str) -> Result<()> {
        // Default to ending the turn unless the model explicitly requests continue.
        self.loop_state = AgentLoopState::Done;

        let message = ChatCompletionRequestUserMessageArgs::default()
            .content(message)
            .build()?
            .into();

        self.messages.push(message);

        let response = self.inner_send_message().await?;

        let message = ChatCompletionRequestAssistantMessageArgs::default()
            .content(response.clone())
            .build()?
            .into();

        self.messages.push(message);

        while let AgentLoopState::Continue = &self.loop_state {
            self.loop_state = AgentLoopState::Done;
            let response = self.inner_send_message().await?;

            let message = ChatCompletionRequestAssistantMessageArgs::default()
                .content(response.clone())
                .build()?
                .into();

            self.messages.push(message);
        }

        self.sender
            .send(HandlerToLooperMessage::TurnComplete)
            .await?;

        Ok(())
    }

    fn set_tools(&mut self, tools: Vec<LooperToolDefinition>) {
        let tools = tools
            .into_iter()
            .map(|t| ChatCompletionTools::Function(t.into()))
            .collect::<Vec<ChatCompletionTools>>();

        self.tools = tools;
    }
}
