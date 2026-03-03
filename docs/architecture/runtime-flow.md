# Runtime Flow

## Overview

This project uses an event-driven loop between UI, `Looper`, handler, and tools.

## Turn Lifecycle

1. CLI reads user input in `main.rs`.
2. `Looper::send()` forwards the message to the active `ChatHandler`.
3. Handler streams events:
   - assistant text
   - reasoning summary text
   - tool call requests
4. `Looper` executes each requested tool through `LooperTools::run_tool()`.
5. Tool results are returned to the handler through a one-shot channel.
6. Handler continues until it sets loop state to `done`.
7. Handler emits `TurnComplete`; UI prints separator and unlocks next prompt.

## API Mode Selection

`Looper::new()` chooses a handler using `LOOPER_API_MODE`:

- `responses` -> `OpenAIResponsesHandler` (default)
- `chat_completions` -> `OpenAIChatHandler`

## Loop State Control

The `set_agent_loop_state` tool is used by the model to control whether another internal iteration should run:

- `state = "continue"` keeps the internal loop going.
- `state = "done"` exits and completes the user turn.
