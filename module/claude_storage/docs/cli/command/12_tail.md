# Command :: 12. `.tail`

### Scope

- **Purpose**: Specify the `.tail` CLI command.
- **Responsibility**: Syntax, parameters, exit codes, and examples for `.tail`.
- **In Scope**: Invocation syntax, accepted parameters, output structure, error conditions.
- **Out of Scope**: Parameter definitions (→ `param/`), type constraints (→ `type/`).

Print the last N conversation turns of the current directory's session. Resolves cwd to its project and, by default, the most recently modified non-agent session, then prints the last 4 turns — no parameters required. Use this for a quick content refresher without running a lookup command first.

**Parameters:** `last::`, `full::`, `compact::`, `path::`, `topic::`

**Exit:** `0` success | `1` argument error | `2` storage read error or project not found

**Syntax:**
```bash
claude_storage .tail
claude_storage .tail last::N
claude_storage .tail full::1
claude_storage .tail compact::1
claude_storage .tail topic::TOPIC
claude_storage .tail path::PATH last::N topic::TOPIC
```

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `last::` (alias `l::`) | Integer | optional | `4` | Number of trailing turns to print; `0` shows all turns |
| `full::` | Boolean | optional | unset | Print every body line instead of folding turns past 8 lines |
| `compact::` | Boolean | optional | unset | One line per turn instead of full bodies |
| `path::` | [`StoragePath`](../type/10_storage_path.md) | optional | cwd | Directory to resolve the project from |
| `topic::` | [`TopicName`](../type/12_topic_name.md) | optional | unset | Session topic suffix to resolve; when omitted, falls back to the most recently modified non-agent session |

**Algorithm (7 steps):**
1. Resolve `path::` (default cwd) to a project ID
2. Resolve the session for `topic::` if given; otherwise fall back to the most recently modified non-agent session in that project
3. Load session entries; exit `2` if the project or session is not found
4. Index every `tool_result` in the session by the `tool_use` id it answers
5. Group entries into turns and drop the ones that render nothing (see Turn Grouping)
6. Take the last `last::` turns (default `4`); `last::0` takes all turns; fewer available turns than requested yields all available
7. Render the session header, then one block per turn, oldest-first

### Turn Grouping

Claude Code writes one JSONL record per content chunk, so a single assistant response routinely occupies several consecutive records that all carry the same `message.id`. `.tail` counts turns, not records:

- **Assistant records join** into one turn when consecutive and sharing a `message.id`.
- **User records always stand alone** — they carry no message id, and consecutive user records are genuinely separate events.
- **A turn that renders nothing is dropped**, so it never consumes a `last::` slot. Two cases produce one: a turn whose only content is `tool_result` blocks (their content is folded onto the `⚙` line of the call they answer), and a turn whose only blocks are empty text or empty thinking.

Consequences worth knowing: `.tail last::4` and `.show last::4` do not select the same amount of history, and turn ordinals in the header (`turns 249-252 of 252`) count displayable turns, whereas the fold hint's `index::` is a raw 1-based entry position for `.show`.

### Output Structure

Default layout — a session header line, then one block per turn:

```text
claude_storage · feed0009 · turns 249-252 of 252 · last 3h ago

── Claude ─────────────────────────────────────────── 17h ago · 16:40 ──
⚙ Bash · git status --short                                  ↳ 3 lines

── You ────────────────────────────────────────────────  3h ago · 09:40 ──
hello
ultrathink
```

- **Session header** — project label, 8-character session id, the displayed turn span out of the session total, and how long ago the newest displayed turn happened.
- **Rule line** — the turn boundary. A rule, not a blank line: message bodies contain blank lines of their own, so whitespace alone can never be an unambiguous separator. It carries the speaker (`You` / `Claude`), relative age, and wall clock. Storage timestamps are UTC; the wall clock is converted to the machine's local timezone before display.
- **Body** — flush-left, so any line can be copied out without stripping a gutter. Never hard-wrapped; the terminal soft-wraps, which keeps copy-paste faithful.
- **Tool calls** — `⚙ Name · summary`, with the result folded onto the same line as a right-aligned `↳ 3 lines` / `↳ 1 line` / `↳ empty` / `↳ error`. The summary is the tool input's most telling string value (`command` for `Bash`, `file_path` for `Read`/`Edit`/`Write`, `pattern` for `Grep`/`Glob`, `status` for `TaskUpdate`, …); paths elide from the front (`…/src/cli/tail.rs`) since the filename identifies them, everything else from the back. Key precedence matters where a tool carries several: `status` outranks `taskId` because `⚙ TaskUpdate · completed` says what happened and `⚙ TaskUpdate · 42` does not. A tool whose input holds no string worth showing — `TaskList` takes none at all, `TodoWrite` and `AskUserQuestion` carry only structured arrays — renders as a bare `⚙ Name`, which measured at 0.7% of tool calls in the local store.
- **Folding** — a turn longer than 8 body lines prints its first 8, then `⋯ N more lines · clg .show session_id::… index::…`. Lifted by `full::1`. Folding never triggers to hide a single line, since the hint would occupy it anyway.
- **Unmodelled blocks** — a content block whose `type` this tool does not model (`image`, and whatever the format grows next) renders as `⧉ image` rather than dropping the record.

Compact layout (`compact::1`) — one row per turn, oldest-first:

```text
claude_storage · feed0009 · turns 249-252 of 252 · last 3h ago

 249   17h  Claude  ⚙ Bash · git status --short ↳ 3 lines
 250   17h  Claude  Confirmed stable and complete: - Zero conflict markers …
 251    3h  You     hello ultrathink
 252    3h  Claude  Hey — the git conflicts from earlier are all resolved a…
```

**Colour:** speakers are coloured by role (user green, assistant cyan), tool lines yellow, chrome dimmed. All of it auto-disables under `NO_COLOR` or when stdout is not a terminal, so piped and test output is plain text with identical structure — every structural element is a glyph, never an escape sequence.

**Examples:**
```bash
# Print the last 4 turns for the current directory (default)
claude_storage .tail

# Print the last 10 turns
claude_storage .tail last::10

# Same, using the `l::` alias
claude_storage .tail l::10

# Print all turns, oldest-first
claude_storage .tail last::0

# Read one long turn in its entirety
claude_storage .tail last::1 full::1

# Scan the last 40 turns, one line each
claude_storage .tail compact::1 last::40

# Print the last 4 turns of a non-default topic
claude_storage .tail topic::work

# Resolve a different directory
claude_storage .tail path::/home/alice/projects/my-app last::6
```

**Notes:**
- Zero-parameter invocation always works: cwd → project → most recently modified non-agent session → last 4 turns (agent sidecar sessions are excluded from this fallback even when they are the newest file — BUG-488)
- `last::0` prints all turns, oldest-first — the full-history equivalent within the resolved session
- Exits `2` when the resolved project or session has no history, matching `.show`'s not-found convention
- Deliberately minimal parameter surface — does not expose `session_id::`, `project::`, or field selection; use `.show` for full inspection, which is what the fold hint links to
- Tool results are indexed across the whole session, not just the displayed window, so a call at the very start of the window still shows what it returned

### Referenced Parameter Groups

| # | Group | Membership | Excluded Params |
|---|-------|------------|-----------------|
| 5 | [Scope Configuration](../param_group/05_scope_configuration.md) | Partial | `scope::` |

### Referenced Parameters

| # | Parameter | Type | Required |
|---|-----------|------|----------|
| 9 | [`path::`](../param/09_path.md) | [`StoragePath`](../type/10_storage_path.md) | optional |
| 17 | [`topic::`](../param/17_topic.md) | [`TopicName`](../type/12_topic_name.md) | optional |
| 25 | [`last::`](../param/25_last.md) | Integer | optional |
| 42 | [`full::`](../param/42_full.md) | Boolean | optional |
| 43 | [`compact::`](../param/43_compact.md) | Boolean | optional |

### Referenced Command Group

Evaluated against `.status` under the strict [command_group](../command_group/readme.md) identity test (same dispatch function, same parameter set) — does not qualify. `tail_routine()` (`src/cli/tail.rs`) has zero cross-calls with `status_routine()` (`src/cli/status.rs:19`). The exit-2 "not found" convention noted in this doc's Notes section is, per `tail_routine()`'s own doc comment, matched against `.status` — not `.show` as this doc's own Notes line states; `show_routine()` (`src/cli/show.rs:41`) never calls `std::process::exit(2)` at all, it returns `Err(ErrorData)` for its not-found case (`src/cli/show.rs:173`). `.tail` and `.status` are the only two routines in the crate that independently call `std::process::exit(2)` (`src/cli/status.rs:45`, `src/cli/tail.rs`) — two separately-written call sites, not a shared function. Parameter sets also differ (`.tail` adds `last::`/`topic::`/`full::`/`compact::`; `.status` adds `show_tokens::`). See [`../command_group/readme.md`](../command_group/readme.md) Evaluated, Not Qualifying for the full analysis.

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 6 | [Quick Context Refresh](../user_story/006_quick_context_refresh.md) | developer |
