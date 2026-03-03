# Adding a Handler

## Overview

Handlers must implement `ChatHandler` from `src/services/chat_handler.rs`.

## Create Handler Implementation

Add `src/services/handlers/<provider>.rs`:

- implement `send_message(...)`
- implement `set_tools(...)`
- implement `set_continue(...)`

Requirements:

- stream assistant text to `HandlerToLooperMessage::Assistant`
- emit tool calls as `HandlerToLooperMessage::ToolCallRequest`
- emit `HandlerToLooperMessage::TurnComplete` at end of turn
- support `set_agent_loop_state` to prevent infinite internal loops

## Export the Module

Update `src/services/handlers/mod.rs` and `src/services/mod.rs` re-exports.

## Wire Into `Looper::new()`

Update `src/looper.rs` API mode selection to construct your handler when selected by env var.

## Confirm Tool Mapping

If your provider uses different tool types, add mapping in `src/mapping/tools/` from `LooperToolDefinition` to provider tool format.
