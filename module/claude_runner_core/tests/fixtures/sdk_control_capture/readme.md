# sdk_control_capture/

Real captured evidence from a live `@anthropic-ai/claude-agent-sdk` v0.3.207 session that
invoked all 25 in-scope `Query` control methods against a real `claude` subprocess. Produced
by task 415 Phase 0 (see `task/claude_runner_core/415_implement_sdk_protocol_bidirectional_control.md`).
Ground truth for the exact wire shapes Phase 2's serde structs must match — not replayed by
any test (this crate's tests use real subprocesses, never mocked/canned responses).

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `argv.json` | Exact argv the SDK invokes the `claude` executable with |
| `wire_stdin.ndjson` | Every `control_request`/`user` message the SDK wrote to stdin, one JSON object per line |
| `wire_stdout.ndjson` | Every message the `claude` subprocess wrote to stdout in response, one JSON object per line |
| `method_results.json` | Per-method JS-side result summary (`{ok, result}` or `{ok: false, error}`) for all 25 `Query` methods |

## Capture method

Generated via a throwaway Node.js shim + driver (not committed — dev-time capture tooling,
see task 415 History) that: (1) tees real stdin/stdout traffic between the SDK and the real
`claude` binary, (2) drives one `query()` session with a streaming-input prompt, (3) calls
each of the 25 in-scope `Query` control methods in sequence, (4) dumps argv + wire NDJSON +
per-method results.

## Redaction

This capture came from a real machine and real user settings, so identifying content was
replaced after capture. Everything below is substituted; all other content is the genuine,
unmodified wire traffic and result shapes.

| Redacted | Replacement |
|----------|-------------|
| `accountInfo()` email and organization | `redacted-user@example.invalid`, `redacted-user's Organization` |
| Absolute home paths (`argv0`, `memoryFiles[].path`, `memory_paths.auto`) | `/home/alice/…` |
| All 28 distinct session / message / request UUIDs (`session_id`, `uuid`, `request_id`, `user_message_id`, `firstUserMessageId`) | Synthetic `cafe00NN-0000-4000-8000-0000000000NN` |
| All 110 user-defined command and skill **names** | `my-skill-001`…`my-skill-098`, `my-plugin:*`, `my-templates:*` |
| Their **descriptions** | `Placeholder description for user-defined entry <name>. (user)` |
| Their non-empty **argumentHints** | `[arg::<value>]` |

Redaction was substitution-only — no field was added, removed, or reordered. JSON/JSONL
structure, key sets, and array lengths are byte-for-byte equivalent to the capture: 25 method
results, 139 commands, 119 skills, 119 skill-frontmatter entries, 47 stdout lines, 24 stdin
lines. Built-in Claude Code command/skill names and descriptions are public and were left
untouched, as were the `(user)` description suffix and `source: "userSettings"` markers — both
are genuine wire behavior that Phase 2 parsing may depend on.

## Key findings evidenced here

- `supportedCommands()`, `supportedModels()`, `supportedAgents()`, `accountInfo()`, and
  `initializationResult()` issue **no wire `control_request` of their own** — `wire_stdin.ndjson`
  shows only one `initialize` request (plus one more for `reinitialize()`). These 5 methods are
  pure accessors over the cached `initialize` control_response, whose `response` object already
  carries `commands`, `agents`, `output_style`, `available_output_styles`, `models`, `account`,
  `pid`, `feedback_survey_config`.
- `reconnectMcpServer()` and `toggleMcpServer()` both return the error
  `"SDK servers should be handled in print.ts"` when targeting an in-process
  (`createSdkMcpServer`) server — confirmed in `method_results.json`.
- `streamInput()` did not resolve within the driver's 15s per-call timeout in this capture
  (`method_results.json` records `TIMEOUT after 15000ms`) — a genuine SDK behavior under the
  driver's specific sequencing (original prompt generator already closed, one probe message
  streamed in), not a capture-tooling defect. Phase 2 should treat `streamInput()`'s ack
  timing as unconfirmed rather than assuming a fast round-trip.
- Wire subtypes use `snake_case` fields even where the JS-facing method is camelCase, e.g.
  `set_max_thinking_tokens` carries `max_thinking_tokens`/`thinking_display`; `rewind_files`
  carries `user_message_id`/`dry_run`; `stop_task` carries `task_id`.
