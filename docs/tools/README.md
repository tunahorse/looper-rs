# Tools

## Overview

Tools are registered in `src/tools/mod.rs` and exposed to handlers through `LooperToolDefinition`.

## Contents

- [Built-In Tools](./built-in-tools.md)
- [Adding a Tool](./adding-a-tool.md)

## Registry

`LooperTools::new()` registers:

- `read_file`
- `write_file`
- `list_directory`
- `grep`
- `find_files`
- `set_agent_loop_state`
