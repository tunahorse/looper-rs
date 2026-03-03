# Conventions

## Overview

Use these conventions when extending handlers, tools, or runtime flow.

## Runtime Contracts

- Keep provider logic in handlers, not in `Looper`.
- Keep local execution logic in `tools/*`.
- Communicate across layers only through typed messages in `types/*`.

## Tool Contract Guidelines

- Always validate/read args defensively.
- Return JSON objects with stable keys.
- Prefer explicit error payloads over panics.

## Streaming and UX

- Send incremental text/thinking updates as they arrive.
- Ensure `ThinkingComplete` is emitted when reasoning output ends.
- Emit `TurnComplete` exactly once per user turn.

## Config Behavior

- Keep env var defaults explicit.
- Document any new env var in `docs/getting-started/configuration.md`.

## Documentation Hygiene

- Keep docs split by concern.
- Link related pages instead of duplicating long explanations.
- Update docs when adding modules, tools, or env vars.
