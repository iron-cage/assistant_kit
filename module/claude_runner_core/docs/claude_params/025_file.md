# file

Download file resources at startup and make them available to the session.

## Type

**CLI** — one or more spec values

## Syntax

```
claude --file <file_id>:<relative_path> [<file_id>:<relative_path> ...]
```

## Default

None

## Description

Downloads file resources (identified by file IDs) at startup and places them at specified relative paths within the session's working directory. The files become available as local files that Claude's tools can read.

Format: `file_id:relative_path`
- `file_id`: The Anthropic Files API file identifier (e.g., `file_abc123`)
- `relative_path`: Where to place the file relative to the working directory

Multiple files can be specified, space-separated. Each `--file` spec downloads one file.

Use cases:
- Providing reference documents to Claude without storing them locally
- Injecting configuration or data files at session start
- Making shared files available across multiple sessions

## Builder API

Use `with_file()` — Repeated-flag: each call adds one file spec.

```rust
use claude_runner_core::ClaudeCommand;

let cmd = ClaudeCommand::new()
  .with_file( "https://example.com/data.csv" )
  .with_message( "Analyze this file" );
```

## Examples

```bash
# Download one file
claude --file "file_abc123:requirements.txt" --print "Implement these requirements"

# Download multiple files
claude --file "file_abc:doc.txt" "file_def:img.png" --print "Analyze these files"
```

## Notes

- Requires Files API access (available with API key auth)
- Files are placed at paths relative to the session working directory
- The files are downloaded once at startup; modifications don't sync back
