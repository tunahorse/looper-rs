# Configuration

## Overview

`looper-rs` is configured through environment variables.

## Required

- `OPENAI_API_KEY`
  - Used by `async-openai` to authenticate requests.

## Optional

- `LOOPER_API_MODE`
  - Selects handler type.
  - Supported values:
    - `responses` (default)
    - `chat_completions`
- `LOOPER_MODEL`
  - Primary model override.
  - Default: `gpt-5.2` if not set.
- `ALCHEMY_MODEL`
  - Secondary model fallback when `LOOPER_MODEL` is not set.

## Example `.env`

```dotenv
OPENAI_API_KEY=your_api_key_here
LOOPER_API_MODE=responses
LOOPER_MODEL=gpt-5.2
```
