# Handlers

## Overview

Handlers implement the `ChatHandler` trait and encapsulate provider streaming behavior.

## Contents

- [OpenAI Responses Handler](./openai-responses.md)
- [OpenAI Chat Completions Handler](./openai-chat-completions.md)

## Trait Contract

`src/services/chat_handler.rs`:

- `async fn send_message(&mut self, message: &str) -> Result<()>`
- `fn set_tools(&mut self, tools: Vec<LooperToolDefinition>)`
- `fn set_continue(&mut self)`
