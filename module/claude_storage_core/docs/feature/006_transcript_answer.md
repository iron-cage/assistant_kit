# Feature: Transcript Answer

### Scope

- **Purpose**: Give a caller holding a live session's conversation id the assistant's text from one turn, read out of the transcript that session is writing, rather than out of whatever the session printed.
- **Responsibility**: Documents the mark/wait/read cycle, what counts as "the answer", why the mark is an entry count rather than a byte or line offset, and why a grace period is part of the contract instead of a caller's retry loop.
- **In Scope**: `transcript_path()`, `transcript_mark()`, `transcript_answer_since()`, the content-block filtering rule, the grace-period semantics, the behaviour when the transcript is absent or shorter than the mark.
- **Out of Scope**: Hosting the session or knowing when its turn ended (→ `claude_daemon_core`), rendering terminal bytes as text (→ `claude_terminal_core`), the CWD → storage path encoding (→ `algorithm/001_path_encoding.md`), entry and content-block parsing (→ `data_structure/001_storage_hierarchy.md`).

### Design

A caller that hosts a live interactive Claude Code session on a terminal has the session's output, and that output is not an answer. It is a picture of an interface: an input box redrawn every frame, a status bar, spinner glyphs, box rules, with the words threaded through them. Rendering those bytes faithfully reproduces exactly that picture — correct, and unusable as an answer.

Filtering the chrome out is not a fix either. The chrome belongs to Claude Code and changes whenever its interface does, so a consumer doing that would be pinned to a TUI layout it does not own.

Claude Code writes the same conversation to `<claude home>/projects/{encoded cwd}/{session id}.jsonl` as it goes. That file is keyed by the conversation id the caller already holds, which is what makes this feature possible at all: the id names the file, and the file holds the words as structured data.

**Three functions, one cycle:**

- `transcript_path( cwd, session_id )` — names the file. Does not check that it exists: a session spawned moments ago has no transcript until its first turn produces one, so absence is the ordinary case rather than an error. `None` means the Claude home could not be resolved or `cwd` would not encode, which to a caller means the same thing — there is nothing to read.
- `transcript_mark( path )` — how many conversation entries the file holds right now. Taken **before** the prompt is sent, so that everything past it is this turn. A file that does not exist is zero entries, not an error.
- `transcript_answer_since( path, mark, grace )` — the assistant text written past `mark`, once there is any. `None` after `grace` elapses.

**Why the mark is an entry count.** Not a byte offset and not a line count. A transcript carries non-conversation lines — `summary`, `mode`, `attachment`, `system` — which `Session::entries()` skips. A mark that counted lines would desynchronise from the entry slice on the very next turn, and the failure would not look like a failure: it would be a plausible answer with the wrong words in it.

**What counts as the answer.** Text blocks from assistant entries, and nothing else. Not thinking blocks, not tool-call parameters, not tool results. Those are how the answer was reached, and Claude Code's own print mode does not print them either. Note that a tool result arrives as a *user* entry carrying a `tool_result` block, so filtering on entry type alone would not have been sufficient — the blocks are walked directly. `Entry::content_text()` flattens every block kind together, which is right for searching and wrong for this.

Several text blocks in one turn are joined with a blank line between them, in the order written, rather than reduced to the first.

**Why the grace period is part of the contract.** The two ends of a turn are not the same instant. The turn is over when the session goes idle and its output stops; the transcript is complete when Claude Code has finished flushing it. The second follows the first closely but not atomically. Waiting a moment for a file that is about to be written beats reporting an empty answer, and putting the wait here rather than in each caller keeps every caller from reinventing the same poll loop with a different interval.

The wait is a blocking poll at 100 ms, which is coarse against a model call that just took seconds and fine against a flush that takes milliseconds.

**Fewer entries than the mark.** The file was replaced underneath the reader — a `--continue` elsewhere, a rewrite. Nothing sensible can be sliced from it, so the result is `None` rather than a guess.

### Algorithm

```text
transcript_path(cwd, session_id):
  1. dir = to_storage_path_for(cwd)?          # encode cwd → <claude home>/projects/{enc}/
  2. return dir / "{session_id}.jsonl"        # never stat'd

transcript_mark(path):
  1. session = Session::load(path)            # a missing file is an error here
  2. return session.entries().len()           # … and 0 to the caller

transcript_answer_since(path, mark, grace):
  1. deadline = now + grace
  2. loop:
     a. if read_answer(path, mark) is Some → return it
     b. if now >= deadline → return None
     c. sleep 100ms

read_answer(path, mark):
  1. session = Session::load(path)             # None on any load error
  2. entries = session.entries()               # non-conversation lines already skipped
  3. fresh = entries[mark..]                   # None if shorter than the mark
  4. text = fresh
       .filter(entry_type == Assistant)
       .flat_map(content_blocks)
       .filter_map(Text { text } => text.trim_end())
       .filter(non-empty)
       .join("\n\n")
  5. return Some(text) if non-empty else None  # empty means "not yet", not "said nothing"
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `../../src/transcript_answer.rs` | Full implementation |
| source | `../../src/entry.rs` | `Entry`, `EntryType`, `ContentBlock` |
| source | `../../src/session.rs` | `Session::entries()` — which lines count as conversation |
| doc | `../algorithm/001_path_encoding.md` | CWD → storage directory path encoding |
| doc | `../feature/004_continuation_detection.md` | `to_storage_path_for()`, shared with this feature |
| doc | `../data_structure/001_storage_hierarchy.md` | Session file structure and JSONL format |
| doc | `../../../claude_terminal_core/docs/feature/001_readable_output.md` | The terminal-rendering path this feature exists instead of |
| doc | `../../../claude_runner/docs/cli/command/14_chat.md` | `clr chat`, the first consumer |

### Sources

| File | Notes |
|------|-------|
| `../../src/transcript_answer.rs` | `transcript_path`, `transcript_mark`, `transcript_answer_since` |
| `../../tests/transcript_answer_test.rs` | CA-1–CA-8 against hand-built transcript fixtures |

### Tests

| File | Notes |
|------|-------|
| `../../tests/transcript_answer_test.rs` | CA-1–CA-8: the text block past the mark (CA-1), thinking/tool blocks excluded including a `tool_result` on a user entry (CA-2), the mark excluding the previous turn (CA-3), a missing transcript (CA-4), several text blocks joined in order (CA-5), a transcript written 150 ms late still found (CA-6), the path shape (CA-7), non-conversation lines neither counted nor printed (CA-8) |
