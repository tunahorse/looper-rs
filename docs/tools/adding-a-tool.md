# Adding a Tool

## Overview

This is the standard workflow for adding a new executable tool.

## Create the Tool File

Add `src/tools/<name>.rs` implementing `LooperTool`:

- `fn tool(&self) -> LooperToolDefinition`
- `async fn execute(&self, args: &Value) -> Value`

Define:

- unique tool name
- clear description
- JSON schema for parameters
- structured JSON result payload

## Register the Tool

In `src/tools/mod.rs`:

1. Add module import:
   - `pub mod <name>;`
   - `pub use <name>::*;`
2. Insert into `LooperTools::new()` registry map.

## Verify Handler Exposure

No extra handler code is needed after registration.

`Looper::new()` calls:

- `tools.get_tools()`
- `handler.set_tools(...)`

Both handlers receive the updated tool set automatically.

## Validate End-to-End

Run:

```sh
cargo run
```

Then prompt the agent to call your new tool and confirm results stream through the UI.
