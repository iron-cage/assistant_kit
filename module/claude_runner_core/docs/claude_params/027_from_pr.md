# from_pr

Resume a session linked to a GitHub Pull Request.

## Type

**CLI** — optional string value

## Syntax

```
claude --from-pr [pr-number-or-url]
claude --from-pr [search-term]
```

## Default

None (no PR-linked session resumed)

## Description

Resumes a Claude Code session that was associated with a specific GitHub Pull Request. Accepts:
- A PR number (e.g., `123`)
- A PR URL (e.g., `https://github.com/org/repo/pull/123`)
- A search term: opens interactive picker filtered by that term
- No argument: opens interactive picker showing all PR-linked sessions

PR-linked sessions are created when Claude Code works on code changes associated with a PR. This allows resuming the exact context from that PR review or implementation session.

## Builder API

Use `with_from_pr()` — Optional-value: `Some(value)` adds `--from-pr value`, `None` adds `--from-pr` without value.

```rust
use claude_runner_core::ClaudeCommand;

// Resume session linked to specific PR
let cmd = ClaudeCommand::new()
  .with_from_pr( Some( "123" ) )
  .with_message( "Continue PR #123 session" );

// Resume most recently linked PR session
let cmd = ClaudeCommand::new()
  .with_from_pr( None::<String> )
  .with_message( "Continue latest PR session" );
```

## Examples

```bash
# Resume by PR number
claude --from-pr 123

# Resume by PR URL
claude --from-pr "https://github.com/myorg/myrepo/pull/456"

# Open interactive picker filtered by term
claude --from-pr "auth"

# Open full picker
claude --from-pr
```

## Notes

- Requires GitHub integration to be configured
- PR-linked sessions are stored with metadata associating them to the PR
- Useful for multi-session PR review workflows
