# Modules

## Overview

This page maps each module area to its responsibility.

## Root Modules

- `src/main.rs`
  - CLI entrypoint, prompt loop, and streamed UI rendering.
- `src/looper.rs`
  - Core orchestrator. Bridges handler events to tools and UI.
- `src/theme.rs`
  - Terminal styles, spinners, and prompt formatting.

## Services

- `src/services/chat_handler.rs`
  - `ChatHandler` trait shared by handlers.
- `src/services/handlers/openai_responses.rs`
  - Responses API implementation with recursive streaming/tool execution.
- `src/services/handlers/openai_completions.rs`
  - Chat Completions implementation with streamed tool-call assembly.

## Tools

- `src/tools/mod.rs`
  - `LooperTool` trait and `LooperTools` registry.
- `src/tools/read_file.rs`
- `src/tools/write_file.rs`
- `src/tools/list_directory.rs`
- `src/tools/grep.rs`
- `src/tools/find_files.rs`
- `src/tools/set_agent_loop_state.rs`

## Types and Mapping

- `src/types/messages.rs`
  - Event/message contracts for handler <-> looper <-> UI.
- `src/types/tool.rs`
  - `LooperToolDefinition` schema used to expose tools to models.
- `src/mapping/tools/openai_responses.rs`
  - Converts `LooperToolDefinition` -> Responses API tool type.
- `src/mapping/tools/openai_completions.rs`
  - Converts `LooperToolDefinition` -> Chat Completions tool type.
