# Built-In Tools

## Overview

This page lists built-in tools, argument shape, and return shape.

## `read_file`

- File: `src/tools/read_file.rs`
- Args:
  - `path: string` (required)
- Returns:
  - success: `{ "path": "...", "content": "..." }`
  - error: `{ "error": "Failed to read ..." }`

## `write_file`

- File: `src/tools/write_file.rs`
- Args:
  - `path: string` (required)
  - `content: string` (required)
- Behavior:
  - Creates parent directories if needed.
  - Overwrites existing files.
- Returns:
  - success: `{ "path": "...", "bytes_written": <number> }`
  - error: `{ "error": "Failed to write ..." }`

## `list_directory`

- File: `src/tools/list_directory.rs`
- Args:
  - `path: string` (optional, default `"."`)
- Behavior:
  - Returns sorted entries.
  - Directories are suffixed with `/`.
- Returns:
  - success: `{ "path": "...", "entries": ["a.rs", "src/"] }`
  - error: `{ "error": "Failed to list ..." }`

## `grep`

- File: `src/tools/grep.rs`
- Args:
  - `pattern: string` (required)
  - `path: string` (optional, default `"."`)
- Behavior:
  - Runs system `grep -rn --include=* <pattern> <path>`.
  - Returns at most 100 lines.
- Returns:
  - `{ "pattern": "...", "path": "...", "matches": [...], "truncated": true|false }`

## `find_files`

- File: `src/tools/find_files.rs`
- Args:
  - `pattern: string` (required)
  - `path: string` (optional, default `"."`)
- Behavior:
  - Runs system `find <path> -path <pattern> -type f`.
  - Returns up to 200 files.
- Returns:
  - `{ "pattern": "...", "path": "...", "files": [...] }`

## `set_agent_loop_state`

- File: `src/tools/set_agent_loop_state.rs`
- Args:
  - `state: "done" | "continue"` (required)
  - `continue_reason: string` (optional)
- Purpose:
  - Signals whether the handler should continue internal loop iterations or end the turn.
