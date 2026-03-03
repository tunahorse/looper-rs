# Messaging

## Overview

The runtime is coordinated through strongly typed messages in `src/types/messages.rs`.

## Handler -> Looper

- `HandlerToLooperMessage::Assistant(String)`
- `HandlerToLooperMessage::Thinking(String)`
- `HandlerToLooperMessage::ThinkingComplete`
- `HandlerToLooperMessage::ToolCallRequest(HandlerToLooperToolCallRequest)`
- `HandlerToLooperMessage::TurnComplete`

`HandlerToLooperToolCallRequest` includes:

- `id`: tool call id from provider
- `name`: tool name
- `args`: JSON args
- `tool_result_channel`: one-shot sender for result

## Looper -> Handler

`LooperToHandlerToolCallResult` includes:

- `id`: tool call id
- `value`: JSON tool output

## Looper -> Interface

- `LooperToInterfaceMessage::Assistant(String)`
- `LooperToInterfaceMessage::Thinking(String)`
- `LooperToInterfaceMessage::ThinkingComplete`
- `LooperToInterfaceMessage::ToolCall(String)`
- `LooperToInterfaceMessage::TurnComplete`

## Why This Matters

This separation keeps provider-specific logic in handlers while keeping tool execution and UI streaming in the core loop.
