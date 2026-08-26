# User Story 032: Hosted Session Chat

### Scope

- **Persona**: Developer working a problem through Claude Code from the shell
- **Goal**: Ask a question, get an answer, get the shell prompt back — and have the next question continue the same conversation instead of starting a new one.

### User Story

> As a developer,
> I want each `clr chat` to read like a print-mode command and land in a session that is still alive when it returns,
> so I can hold one conversation across many shell commands — piping, scripting, and interleaving other work between turns — without paying for the context to be rebuilt every time.

### Acceptance Criteria

- **AC-1 (Answer, not terminal):** `clr chat "<MESSAGE>"` prints the assistant's words on stdout. Not a rendering of Claude Code's interface — no box rules, no input box, no status bar, no spinner frames. Progress notes and warnings go to stderr, so `clr chat … | …` carries the answer alone.
- **AC-2 (Continuity):** a second `clr chat` in the same directory reaches the same session, so it can answer questions about the first. This is the difference between it and `clr ask`.
- **AC-3 (No setup step):** the first `clr chat` in a fresh environment starts the daemon and the session itself. `clr daemon start` exists but is never a prerequisite.
- **AC-4 (The session survives):** when `clr chat` returns, the session it talked to is still hosted. `clr sessions` lists it with the conversation id, the pid, whether a turn is in flight, and the directory it runs in.
- **AC-5 (Addressing):** `clr chat --session <ID>` talks to a session by conversation id wherever it lives; `clr chat --dir <PATH>` selects one by directory. The id `clr sessions` prints is the id `--session` takes, never abbreviated.
- **AC-6 (Any length of prompt):** a prompt is delivered and answered regardless of its length — a long one is not silently left sitting in the session's input box.
- **AC-7 (Asking is not doing):** `clr sessions` never starts a daemon. With none running it explains on stderr, prints `[]` under `--json`, and exits 0 — "nothing is hosted" is a complete answer to the question asked.
- **AC-8 (Idempotent lifecycle):** `clr daemon status`, `start`, and `stop` describe the state on **return**, not what the call did. Both transitions are safe to run twice, which is what makes `clr daemon status || clr daemon start` work.
- **AC-9 (One daemon):** at most one daemon runs at a time, enforced by an advisory lock rather than a PID file. A second one that loses the race exits 0 quietly — whoever started it wanted a daemon running, and there is one.
- **AC-10 (Outlives the shell):** the daemon and its sessions survive the shell that started them — their own process group, no controlling terminal, reparented to init. Closing the terminal does not end the conversation.
- **AC-11 (The terminal is still reachable):** `clr chat --raw` prints the session's terminal bytes, interface and all. The default stops showing the chrome; it does not put it out of reach.
- **AC-12 (A cut-short answer is not a failure):** an answer ended by `--timeout` exits 0 with a warning on stderr. The prompt was delivered and the session is still working; `clr chat` again prints the rest.
- **AC-13 (Argument errors cost nothing):** a wrong argument to `clr chat` exits 1 before any daemon is contacted, and leaves no process and no socket behind.
- **AC-14 (Teardown):** `clr daemon stop` takes every hosted session down with it, and returns only once nothing is answering the socket — acknowledged is not stopped.

### Primary Flags

| Flag | Role |
|------|------|
| `<MESSAGE>` | The prompt. One argument — quote it |
| `--dir <PATH>` | Which directory's session to talk to |
| `--session <ID>` | Talk to a session by conversation id, wherever it lives |
| `--timeout <SECS>` | How long to wait for the answer before returning what arrived |
| `--raw` | Print the session's terminal bytes instead of the answer its transcript recorded |
| `--json` | (`sessions`) print the daemon's own list verbatim instead of a table |

### Examples

```sh
# Ask something. Starts the daemon and a session if neither exists yet.
clr chat "what does this crate do?"

# Ask again — same directory, so the same session, so it remembers.
clr chat "and what depends on it?"

# The answer alone, into a file
clr chat "list the public types, one per line" > types.txt

# What is hosted right now?
clr sessions

# Talk to one of them by id
clr chat "carry on" --session "$( clr sessions --json | jq -r '.[0].session_id' )"

# A session somewhere else
clr chat "summarise the tests" --dir ~/work/other-project

# A long job, and a willingness to wait for it
clr chat "refactor the parser" --timeout 1800

# The bytes as the terminal sent them — interface and all
clr chat "draw me a table" --raw

# Start one explicitly if it is not already running, then take it all down
clr daemon status || clr daemon start
clr daemon stop
```

### Related Commands

| Command | Role |
|---------|------|
| `chat` | Primary command for this user story — one prompt, one answer, session intact |
| `sessions` | Lists what is hosted, and prints the ids `--session` takes |
| `daemon` | Lifecycle of the process hosting them: `status`, `start`, `stop`, `log` |
| `ask` / `run` | The same shape of interaction with no session surviving it |
| `ps` | Every Claude Code process on the machine, hosted or not |

### Related Doc Instances

| File | Relationship |
|------|--------------|
| [`../command/14_chat.md`](../command/14_chat.md) | `clr chat` command reference |
| [`../command/15_sessions.md`](../command/15_sessions.md) | `clr sessions` command reference |
| [`../command/13_daemon.md`](../command/13_daemon.md) | `clr daemon` command reference |
| [`../../../../claude_daemon_core/docs/feature/001_single_instance.md`](../../../../claude_daemon_core/docs/feature/001_single_instance.md) | The lock behind AC-9 |
| [`../../../../claude_daemon_core/docs/feature/006_serving_clients.md`](../../../../claude_daemon_core/docs/feature/006_serving_clients.md) | The socket, and the submit gap behind AC-6 |
| [`../../../../claude_daemon_core/docs/feature/004_session_output.md`](../../../../claude_daemon_core/docs/feature/004_session_output.md) | The cursors AC-1's reading loop trades in |

### Related User Stories

| # | Title | Relationship |
|---|-------|--------------|
| 001 | [Interactive REPL](001_interactive_repl.md) | The same real interactive session, attached to the caller's terminal instead of hosted |
| 002 | [Print Mode Capture](002_print_mode_capture.md) | The UX this story borrows — prompt in, answer out — without the session surviving |
| 026 | [Session Listing](026_session_listing.md) | `clr ps` finds every Claude Code process; `clr sessions` lists only what this daemon owns |
| 025 | [Session Concurrency Gate](025_concurrency_gate.md) | Bounds how many sessions exist at once; this story keeps each one alive longer |
