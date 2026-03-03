# Architecture

## Overview

This section documents how the runtime loop, handlers, and tools coordinate.

## Contents

- [Runtime Flow](./runtime-flow.md)
- [Modules](./modules.md)
- [Messaging](./messaging.md)

## High-Level Shape

- `main.rs` runs the CLI loop and renders streamed UI events.
- `looper.rs` orchestrates handler events and tool execution.
- `services/handlers/*` implement model-provider streaming logic.
- `tools/*` define executable local tools and JSON schemas.
- `types/*` define message and tool definition contracts.
- `mapping/*` maps local tool definitions into OpenAI SDK tool types.
