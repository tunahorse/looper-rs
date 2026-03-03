# Running

## Overview

Use this page to run the CLI and switch handler API modes.

## Start the CLI

```sh
cargo run
```

## Runtime Behavior

1. The terminal UI starts and displays a prompt.
2. Your input is sent to the `Looper`.
3. The handler streams assistant output, thinking summaries, and tool calls.
4. Tool calls execute inside `LooperTools`.
5. The turn completes when the handler emits `TurnComplete`.

## Switching API Modes

Run with Responses API mode:

```sh
LOOPER_API_MODE=responses cargo run
```

Run with Chat Completions mode:

```sh
LOOPER_API_MODE=chat_completions cargo run
```
