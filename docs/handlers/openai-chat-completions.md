# OpenAI Chat Completions Handler

## Overview

Implements streaming chat completions with incremental tool-call assembly and recursive continuation.

## Source

`src/services/handlers/openai_completions.rs`

## Behavior

- Uses `CreateChatCompletionRequestArgs` and `chat().create_stream(...)`.
- Accumulates assistant text deltas from `choice.delta.content`.
- Reconstructs tool calls from chunked `choice.delta.tool_calls`.
- On `FinishReason::ToolCalls`, dispatches all collected tool requests to `Looper`.
- Appends assistant tool-call message plus tool response messages to chat history.
- Recursively calls `inner_send_message()` until no tool calls remain.

## Loop State

- Starts each user turn in `AgentLoopState::Continue`.
- Updates to `Done` when `set_agent_loop_state` is called with `state = "done"`.
- Continues internal turns while state remains `Continue`.

## Model Resolution

Model selection order:

1. `LOOPER_MODEL`
2. `ALCHEMY_MODEL`
3. fallback `gpt-5.2`
