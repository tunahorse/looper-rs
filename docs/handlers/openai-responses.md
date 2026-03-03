# OpenAI Responses Handler

## Overview

Implements the OpenAI Responses API flow with streaming deltas and recursive tool-result continuation.

## Source

`src/services/handlers/openai_responses.rs`

## Behavior

- Uses `CreateResponseArgs` and `responses().create_stream(...)`.
- Streams assistant text via `ResponseOutputTextDelta`.
- Streams reasoning summaries via `ResponseReasoningSummaryTextDelta`.
- Captures function calls from `ResponseOutputItemDone`.
- Sends tool requests to `Looper` through `HandlerToLooperMessage::ToolCallRequest`.
- Waits for tool outputs, then sends `FunctionCallOutput` items in the next `inner_send_message()` recursion.
- Tracks `previous_response_id` for server-side conversation continuity.

## Loop State

- Initializes each user turn to `AgentLoopState::Continue`.
- Watches for `set_agent_loop_state` tool calls.
- Ends turn when tool sets `state = "done"`.

## Model Resolution

Model selection order:

1. `LOOPER_MODEL`
2. `ALCHEMY_MODEL`
3. fallback `gpt-5.2`
