# Data Structure: Session Context State

### Scope

- **Purpose**: Reconstruct what a session's context currently holds — deferred tools, agent and skill rosters, remaining token budget, background tasks — none of which the session file states directly.
- **Responsibility**: Documents `SessionContextState`, the `ContextFold` accumulator, the resolution rule for each field, and the incremental-read contract that makes following a live session safe.
- **In Scope**: Delta accumulation semantics, first-wins/last-wins/replace rules per field, sidechain exclusion, degradation policy, byte-offset resume.
- **Out of Scope**: The line schema being folded (→ `data_structure/003_session_event.md`), token *usage* accounting (→ `SessionStats` in `../../src/stats.rs`), the harness's static system prompt, which never appears in the JSONL.

### Abstract

Most of a session's context arrives as **deltas, not snapshots**. A `deferred_tools_delta` line says which tools were added and which removed; it never says what the set now holds. The current state therefore exists nowhere in the file — it has to be accumulated by replaying every delta in order. `ContextFold` does that replay; `SessionContextState` is the result.

The distinction matters because a reader that samples one line gets a change, not a state. Only the fold answers "what is loaded right now".

### Structure

**ContextFold** — the accumulator. Holds the state built so far plus the byte offset up to which the file has been consumed.

- `apply(&SessionEvent)`: fold one event.
- `read_file(&Path) -> Result<usize>`: read every whole line added since the last call, fold each in, return how many were applied. Callable repeatedly against a growing file.
- `state()` / `into_state()`: borrow or take the result.
- `offset()`: the resume point. Only whole lines are counted, so it never points into the middle of one; persist it to resume across process restarts.

**SessionContextState** — the accumulated context. Grouped by how each field resolves:

| Resolution | Fields |
|-----------|--------|
| First-wins | `session_id`, `cwd` |
| Last-wins | `version`, `mode`, `permission_mode`, `title`, `last_prompt_uuid`, `date`, `task_reminder_items` |
| Last-wins, parseable only | `tokens_remaining` |
| Delta-accumulated set | `deferred_tools`, `agent_types`, `mcp_servers`, `attached_files` |
| Replaced wholesale | `pending_mcp_servers`, `skills_available`, `skills_reported_count`, `allowed_tools` |
| Upsert by key | `tasks` |
| Append, first occurrence only | `skills_invoked` |
| Tallies | `counters` |

**TaskState** — a background task's most recent reported `task_type`, `status`, `description`, `output_file_path`.

**EventCounters** — `lines_read`, `lines_skipped`, `sidechain_events`, `user_messages`, `assistant_messages`, `compactions`, `queued_commands`, `system_subtypes`, and the two `unmodelled_*` maps.

### Design Decisions

- **`cwd` is first-wins, `version` is last-wins.** `cwd` identifies where the session started and must not drift as attachments carry their own paths. `version` identifies what wrote the newest line and *must* drift — a session resumed after an upgrade should report the version now in use, not the one it began under.
- **Within one delta, additions beat removals.** A name appearing in both `removedNames` and `addedNames` resolves to present. Removals are applied first so the addition, being the newer fact, lands last. Leaving the order undefined would make the result depend on iteration order.
- **`readdedNames` folds in like an addition.** The distinction between a first deferral and a re-deferral matters to the event schema (`003`) but not to the resulting set — both mean the tool is deferred now.
- **`isInitial` clears before applying.** An initial listing is a full snapshot. Merging it would leave agents from a prior listing that are no longer offered.
- **A skill listing replaces rather than merges**, for the same reason. Its self-reported count is retained alongside the names so `skills_truncated()` can report a disagreement — the names alone cannot say what is missing.
- **A skill invoked twice is recorded once.** The harness injects a skill's text on first invocation; a repeat adds nothing to what is in context, so counting it twice would overstate the context's contents.
- **Sidechain lines are counted, never folded.** A subagent's context is its own. Letting its roster deltas through would corrupt the main conversation's view of what it has loaded. `counters.sidechain_events` keeps the exclusion visible rather than silent.
- **Unmodelled kinds are counted by name.** A newer Claude Code's added line or attachment kind lands in `unmodelled_kinds` / `unmodelled_attachments` rather than vanishing, and `has_unmodelled()` reports the condition. Silence would read as "nothing was there" when the truth is "this build's schema is behind".

### Token Accounting

`tokens_remaining` is the only token figure here, and it is deliberately not accompanied by any usage figure.

The remaining budget is the harness's own number, injected as prose in a `total_tokens_reminder`. It is **not** derivable by summing usage — every turn re-sends the whole conversation, so those sums exceed the window many times over in a long session and measure cost, not fullness.

Usage is a different matter — `Session::stats()` already computes it, deduplicating by `message.id` and splitting cache reads from cache writes. Recomputing it here would be a second, divergent implementation of one sum. A caller wanting both halves asks each owner for its own:

```rust,ignore
let occupied = session.stats()?.last_context_tokens;
let remaining = session.context_state()?.tokens_remaining;
```

**On the static system prompt.** Its *text* never appears in the JSONL, but its *cost* does — `last_context_tokens` is one call's whole billed prompt, tools and system prompt included. So the two figures above bracket the model's context window (`occupied + remaining`), which is otherwise recorded nowhere. What remains genuinely unavailable from the transcript is the *split* between fixed overhead and conversation: knowing that a session occupies 30k tokens does not say how much of it was spent before the first word.

### Incremental Reads

`read_file` is safe to call against a session that is still being written. Four cases are handled rather than reported as failures:

| Condition | Behaviour | Why |
|-----------|-----------|-----|
| Trailing line with no newline | Left unconsumed; re-read whole next call | A write in progress. Parsing it would fold a half-written event; consuming it would mean never seeing the whole one. |
| Line not valid UTF-8, or not a session line | Skipped, counted in `lines_skipped` | One bad line must not discard the rest of the file — the same per-line skip policy `stats()` and `search()` use. |
| Blank line | Consumed, not counted as a skip | Whitespace is not a malformed record. |
| File shorter than the offset | Fold restarts from zero with fresh state | The file was replaced, not appended to. Continuing would skip the new file's opening lines and mix state from two sessions. |

Only I/O failure — the file cannot be opened, measured, or sought — surfaces as an error.

### Edge Cases

- **Empty file**: folds to empty state, offset stays at zero, no error. This is the normal state of a session just created.
- **Session with no `skill_listing` line**: `skills_available` is empty and `skills_reported_count` is zero, indistinguishable from an empty listing. The distinction has no consumer, so no flag is carried for it.
- **`tokens_remaining` never set**: `None`, meaning the session carried no parseable reminder — not "zero tokens left".
- **Repeated `read_file` with nothing appended**: applies zero events, leaves offset unchanged.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `../../src/context.rs` | `SessionContextState`, `ContextFold`, `TaskState`, `EventCounters` |
| source | `../../src/event.rs` | `SessionEvent` — the line schema being folded |
| source | `../../src/session.rs` | `Session::context_state()` — one-shot convenience over the fold |
| source | `../../src/stats.rs` | `SessionStats` — owns token *usage*, which this type defers to |
| test | `../../tests/context_test.rs` | Delta accumulation, incremental read, and degradation tests |
| doc | `../data_structure/003_session_event.md` | The line and attachment taxonomy |

### Sources

| File | Notes |
|------|-------|
| Session JSONL files | Delta vs snapshot semantics inferred from observed attachment sequences |
