# Error: Context Limit Reached

### Scope

- **Purpose**: Document the family of errors and blocking messages Claude Code emits when a conversation or request exceeds the model's context window.
- **Responsibility**: Describe the in-session blocking message, the underlying API error variant, the auto-compact thrash variant, and all recovery paths.
- **In Scope**: In-session blocking UI message, HTTP 400 `invalid_request_error` for token overflow, autocompact thrash error, request-too-large client check.
- **Out of Scope**: Rate limit errors (→ `error/001_rate_limit_reached.md`); request timeout behavior (→ `error/004_request_timed_out.md`).

### Abstract

Context limit errors surface in three distinct forms depending on where the limit is hit:

**1. In-session blocking message** (interactive mode, printed to UI):
```
Context limit reached · /compact or /clear to continue
```
Claude Code halts input processing. No exit code is emitted; the session stays open waiting for `/compact` or `/clear`.

**2. API-level overflow error** (non-interactive or after failed auto-compact):
```
API Error: 400 {"type":"error","error":{"type":"invalid_request_error","message":"input length and `max_tokens` exceed context limit: 197202 + 21333 > 200000, decrease input length or `max_tokens` and try again"},"request_id":"req_011..."}
```
The request is rejected before Claude sees it. The exact token counts appear in the message. Exit code: non-zero.

**3. Client-side size check** (before any API call):
```
Request too large (max 20MB). Double press esc to go back and try with a smaller file.
```
The binary checks payload size locally. Triggered by attaching very large files.

**4. Autocompact thrash** (when auto-compact loops):
```
Autocompact is thrashing: the context refilled to the limit...
```
Auto-compaction succeeded but a tool output or included file immediately refilled context repeatedly.

### Trigger Conditions

- **Long conversation**: A conversation accumulates enough turns, code, and tool output to approach the model's token window (200 000 tokens for Claude claude-sonnet-4-6).
- **Large `max_tokens` setting**: A high `--max-tokens` value reduces the headroom for input; the sum `input_tokens + max_tokens > context_limit` triggers the 400 error even with a moderate conversation.
- **Large file attachment**: A file or glob expansion produces a payload over 20 MB before API submission; the binary rejects it client-side.
- **Autocompact loop**: A file read via a tool (e.g., `cat` on a large log) immediately replenishes context after compaction, causing thrashing.

### Recovery

**Interactive session:**
1. Type `/compact` to summarize the conversation and free context space. Claude Code replaces the full history with a condensed summary and continues.
2. Type `/clear` to discard the entire conversation history and start fresh in the same session.
3. Set `CLAUDE_CODE_BLOCKING_LIMIT_OVERRIDE=197000` (or lower) in the environment to trigger compaction before the hard limit is hit.

**API-level 400 (non-interactive / script):**
1. Reduce the `--max-tokens` value passed to `claude` (or via `clr`), or shorten the input.
2. Break the task into smaller subtasks, each in a fresh session (`--new-session`).
3. Use `clr --new-session "..."` to start without prior context.

**Large file attachment:**
1. Avoid attaching files over ~20 MB. Pre-filter or truncate large logs before passing them to the session.

**Autocompact thrash:**
1. Avoid reading very large files through Claude's tool use in the same session where the context is nearly full.
2. Use `/clear` and restart with a fresh session dedicated to the large-file operation.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| error | [error/001_rate_limit_reached.md](001_rate_limit_reached.md) | Rate-limit error — distinct cause (quota, not token size) |
| error | [error/004_request_timed_out.md](004_request_timed_out.md) | Timeout error — occurs when context is large but not over limit |
| source | `../../module/claude_runner/src/main.rs` | `--max-tokens` flag that affects the input+max_tokens sum |
