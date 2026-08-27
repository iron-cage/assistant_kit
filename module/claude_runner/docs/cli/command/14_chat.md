# CLI Command: chat

### Description

Send one prompt to a hosted interactive session and print the answer. From the outside
it behaves like print mode — prompt in, answer out, shell prompt back — but the session
it talked to is still alive afterwards, still holding the conversation, so the next
`clr chat` in the same directory continues it instead of starting over.

This is the command the daemon exists for. Everything else in the daemon stack is
machinery underneath it.

-- **Parameters:** `<MESSAGE>`, `--dir <PATH>`, `--session <ID>`, `--timeout <SECS>`, `--raw`
-- **Exit Codes:** 0 (an answer was printed, complete or not) | 1 (nothing could be sent)
-- **Forms:** transition (starts a daemon and possibly a session), then query

### Syntax

```sh
clr chat "<MESSAGE>"
clr chat "<MESSAGE>" --dir <PATH>
clr chat "<MESSAGE>" --session <ID>
clr chat "<MESSAGE>" --timeout <SECS>
clr chat "<MESSAGE>" --raw
clr chat help
```

### Parameters

| # | Name | Required | Default | Purpose |
|---|------|----------|---------|---------|
| 1 | `<MESSAGE>` | Yes | — | The prompt. One argument — quote it |
| 2 | `--dir <PATH>` | No | current directory | Which directory's session to talk to |
| 3 | `--session <ID>` | No | resolved from `--dir` | Talk to this session by conversation id |
| 4 | `--timeout <SECS>` | No | `300` | Give up waiting for the answer after this long |
| 5 | `--raw` | No | off | Print the session's terminal bytes instead of the answer the transcript recorded |

`<MESSAGE>` is deliberately a single argument rather than a trailing catch-all. Two bare
words are rejected with a suggestion to quote, because the alternative failure — chatting
about only the first word — succeeds, costs a model call, and answers a question nobody
asked.

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | The prompt was delivered and whatever came back was printed |
| 1 | The arguments were wrong, the daemon could not be started, or the session would not take the message |

An answer cut short by `--timeout` still exits 0, with a warning on stderr. The prompt
was delivered and the session is still working on it; `clr chat` again prints the rest.
Failing there would suggest the work was lost, which it was not.

**Algorithm — resolving the session (5 steps):**
1. `--session <ID>` given → use it, whatever directory it is in and whether or not it is busy. The caller named it.
2. Otherwise ask the daemon for its sessions and match on canonicalised working directory.
3. A match → use it. This is what makes two chats in a row continue one conversation.
4. No match → `spawn` a session in that directory, then wait 3 seconds for its interface to finish drawing before sending anything.
5. Either way, keep the session's **own** working directory alongside its id — that directory is half of the transcript's address, and `--session` can name a session somewhere else entirely.

**Algorithm — collecting the answer (6 steps):**
1. `send` returns the output cursor from immediately *before* the write, so reading starts exactly at this prompt.
2. `read` from that cursor every 100ms, accumulating raw text and advancing the cursor.
3. A read that reports `missed > 0` sets a flag — output was evicted before it could be fetched, and the answer has a gap in it.
4. A read that reports `ended` returns immediately: the session's terminal is closed and nothing more will ever arrive.
5. Otherwise, return once output has been quiet for 8 consecutive polls **and** the daemon reports the session not busy.
6. The deadline from `--timeout` ends the loop regardless, with a warning.

**Algorithm — printing (5 steps):**
1. Before `send`, count the conversation entries already in the session's transcript. That count is the mark: everything past it is this turn.
2. `--raw` → the accumulated terminal bytes, exactly as they arrived. The transcript is not consulted at all, so no time is spent looking for a nicer answer nobody asked for.
3. Otherwise → read the transcript entries past the mark, keep the assistant ones, keep their **text** blocks only, and join them. Thinking blocks, tool calls, and tool results are how the answer was reached, and print mode does not print those either.
4. A transcript that is not there yet is polled for up to 5 seconds — the turn ends when the session goes idle and quiet, which is slightly before Claude Code has finished flushing the file. Still nothing → fall back to [`to_plain_text`](../../../../claude_terminal_core/docs/feature/001_readable_output.md) over the terminal bytes, and say so on stderr.
5. Warnings about gaps, session end, and timeouts go to stderr, so a caller redirecting stdout gets the answer alone. The gap warning is suppressed when the answer came from the transcript — `missed` describes an eviction from the terminal's ring buffer, which is not a gap in a file.

### Examples

```sh
# Ask something. Starts the daemon and a session if neither exists yet.
clr chat "what does this crate do?"

# Ask again — same directory, so the same session, so it remembers.
clr chat "and what depends on it?"

# A session somewhere else
clr chat "summarise the tests" --dir ~/work/other-project

# A specific session, by the id `clr sessions` prints
clr chat "carry on" --session 4f2c8a1e-...

# The bytes as the terminal sent them — interface and all, not just the answer
clr chat "draw me a table" --raw

# A long job, and a willingness to wait for it
clr chat "refactor the parser" --timeout 1800
```

### Notes

**Knowing when the answer is finished is the hard part.** A hosted session is a terminal
application: output arrives continuously, including while it is thinking, and nothing in
the stream says "done". Two independent signals must agree, because each alone is wrong
in a way the other is not.

The session's self-reported status is too eager on its own — it is written by another
process, so for a moment after `send` the session is still recorded idle from *before*
the prompt arrived, and a client trusting it would return having printed nothing. Silence
is too eager on its own in the opposite direction — a session waiting on a slow tool call
is quiet for seconds at a time without being finished. Idle *and* quiet is neither: the
status lag is covered by the quiet requirement, because output is streaming during it,
and the mid-turn pause is covered by the idle requirement, because a session waiting on a
tool is recorded busy.

**Knowing *when* is not knowing *what*.** The two signals above settle when the turn
ended. They say nothing about what the answer was, and the terminal — where the words
physically arrived — is a bad place to ask, because what arrived is a picture of Claude
Code's interface rather than a message: input box, status bar, spinner frames, box rules,
and the answer somewhere among them. Rendering those bytes faithfully produces exactly
that picture. Correct, and unusable as the output of a command promising prompt in,
answer out.

Filtering the chrome would not fix it either, because the chrome belongs to Claude Code
and changes whenever its interface does — a `clr` release would be pinned to a TUI layout
it does not own. So the answer is read from the session's own transcript instead
(`<claude home>/projects/<encoded cwd>/<session id>.jsonl`), as structured data, keyed by
the same conversation id the daemon already holds. The pty keeps doing the thing only a
pty can do — carry a real interactive session, statefully, across turns — and the
transcript answers the question the pty is bad at: what did it actually say.

**`idle` only means "finished" because the daemon earns it.** A session waiting on an
outstanding background task also reports `idle` — unless it was started with
`CLAUDE_CODE_BG_TASKS_REPORT_RUNNING=1`, which the daemon's spawner sets. Requiring
quiet as well is what keeps this command correct if that ever stops being true.

**Auto-starting the daemon is right here and wrong for `sessions`.** A client asking to
talk to a session wants a session, and the daemon is how it gets one — an extra setup
step would be ceremony, not safety. `clr sessions` asks a *question*, and a question that
starts a process to answer itself has changed the thing it was asking about.

**Argument errors start nothing.** Parsing happens before the daemon is touched, so a
typo costs nothing and leaves no process behind.

**A session that never registers gets a hint, not just a failure.** The daemon can only
report that no conversation id arrived — it cannot see why, because the reason is on a
terminal it does not read. In practice the usual cause is a `claude` that came up in a
first-run prompt (theme picker, trust prompt) and is sitting there waiting to be
answered, having never got as far as opening a conversation. That is invisible from here
and unfixable from here, but it is fixable in one step, so the message says which step.
The child itself does not survive the failure — the daemon kills it before reporting, or
it would hold a terminal forever with nobody able to address it.

**The prompt is sent separately from the spawn**, even though `spawn` accepts one inline.
The daemon delivers an inline prompt the instant registration completes, which is earlier
than the interface is ready to be typed into.

**Error messages:**
- `Error: 'clr chat' needs a message` — followed by a pointer to `clr chat help`.
- `Error: unexpected extra argument "<token>"` — followed by the suggestion to quote.
- `Error: unknown option "<token>" for 'clr chat'` — followed by a pointer to help.
- `Error: --timeout wants a whole number of seconds, got "<value>"`.
- `Error: <reason>` from `ensure_running`, followed by the daemon log path.
- `Error: the session would not start: <reason>` — the `spawn` request failed. When the reason is `never registered a conversation id`, a hint follows: run `claude` once in this environment and answer any first-run prompts.
- `Error: the daemon started a session but did not name it` — `spawn` succeeded with an empty conversation id.
- `Error: the session would not take the message: <reason>` — the `send` request failed.

**Warnings (stderr, exit still 0):**
- `Warning: some output was dropped before it could be read` — the answer has a gap. Only when the answer came from the terminal; a transcript read has no ring buffer to overflow.
- `Note: the session ended while answering.`
- `Warning: gave up waiting` — followed by a note that the session is still running.
- `Note: the answer could not be read from the session transcript — showing the terminal instead.` — the fallback fired, and the output above is a terminal rather than a message.

### Referenced Command Group

Evaluated against `query` under the strict [command_group](../command_group/readme.md)
identity test (same dispatch function, same parameter set) — does not qualify.
`dispatch_chat()` (`src/cli/chat.rs`) and `dispatch_query()` (`src/cli/query.rs`) both
send a message to a running Claude Code process, but through different daemons, keyed
differently (conversation id versus PID), over different protocols, with no shared
parameters and no cross-calls.

### Related Commands

| # | Command | Relationship |
|---|---------|--------------|
| 1 | [`sessions`](15_sessions.md) | Lists the sessions this command talks to, and prints the ids `--session` takes |
| 2 | [`daemon`](13_daemon.md) | The lifecycle of the process hosting them; `chat` starts it, `daemon stop` ends it |
| 3 | [`ask`](05_ask.md) | The same shape of interaction in print mode — no session survives it |
| 4 | [`query`](10_query.md) | Control-plane messages to a PID-addressed session, not conversation |

### Referenced Parameter Groups

None. `chat` takes its own flags and forwards nothing to `claude`.

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 32 | [032_hosted_session_chat.md](../user_story/032_hosted_session_chat.md) | Developer |

---

**Category:** Session management
**Complexity:** 9
**API Requirement:** Write
**Idempotent:** No — every invocation sends a prompt and consumes a turn
**Risk Level:** Medium
