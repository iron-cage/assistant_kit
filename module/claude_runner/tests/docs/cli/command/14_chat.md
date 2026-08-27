# Test: `chat`

Integration test planning for the `chat` command. See [command/14_chat.md](../../../../docs/cli/command/14_chat.md) for specification.

`chat` sends a prompt to a hosted session and prints the answer. Tests verify the help
surface, every way the arguments can be wrong, the typo guard on the subcommand itself,
the ordering guarantee that argument errors are settled before a daemon is started to
discover them, and — separately — that the answer printed is the answer rather than a
picture of the terminal it arrived on.

Two files, because the two concerns fail differently. A wrong argument fails loudly and
costs nothing; a wrong *read* produces a plausible-looking answer with the wrong words
in it, which is why the second set uses real transcript fixtures rather than argument
strings.

| File | Cases | Subject |
|------|-------|---------|
| `chat_command_test.rs` | CH-1–CH-10 | The argument surface, and the order things happen in |
| `claude_storage_core/tests/transcript_answer_test.rs` | CA-1–CA-8 | Reading a turn's answer out of the session transcript |

The second file is not in this crate. The reading rule is a fact about how Claude Code
writes storage, so it lives with the parser it uses; `clr chat` consumes it through
`claude_storage_core::transcript_answer_since`. It is listed here anyway because this
command is what the cases exist for.

## What is deliberately not tested here

A chat that actually completes. It needs a real `claude` on `PATH`, answering on a real
terminal, over a real model call — that is an end-to-end concern rather than a CLI one,
and it would make this file's runtime depend on a network round-trip and a working
subscription. That round trip is exercised by hand instead; see `tests/manual/readme.md`.

The layers under it are tested where they live, against real implementations rather than
mocks: the terminal in `claude_pty_core`, the spawn/send/read cycle in
`claude_daemon_core`'s `serve_test.rs` (real socket, real client, real PTY-attached
children), the rendering of what comes back in `claude_terminal_core`'s `render_test.rs`,
and the answer read out of the transcript in `claude_storage_core`'s
`transcript_answer_test.rs`.

What remains for these files is everything `chat` decides on its own: the argument
surface, the order it does things in, and where it gets the words it prints.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| CH-1 | `clr chat help` documents every option | Documentation |
| CH-2 | No message → exit 1, says a message is needed | Validation |
| CH-3 | Unknown option → exit 1, names it | Validation |
| CH-4 | `--timeout soon` → exit 1, quotes the bad value | Validation |
| CH-5 | `--session` with nothing after it → exit 1 | Validation |
| CH-6 | Two bare arguments → exit 1, suggests quoting | Validation |
| CH-7 | An argument error starts no daemon | Ordering |
| CH-8 | `clr chatt` (typo) → exit 1, suggests `chat` | Typo guard |
| CH-9 | The onboarding hint's trigger phrase still matches the daemon's error | Contract |
| CH-10 | `--session <ID>` fails at `send`, not at `spawn` | Addressing |
| CA-1 | An assistant text block past the mark is the answer | Answer reading |
| CA-2 | Thinking and tool blocks are excluded | Answer reading |
| CA-3 | The mark excludes the previous turn | Answer reading |
| CA-4 | A missing transcript falls back rather than failing | Fallback |
| CA-5 | Several text blocks are joined in order | Answer reading |
| CA-6 | The grace period waits for a transcript still being written | Timing |
| CA-7 | `transcript_path` names `<encoded cwd>/<session id>.jsonl` | Addressing |
| CA-8 | Non-conversation lines are neither counted nor printed | Answer reading |

## Test Coverage Summary

- Documentation: 1 test (CH-1)
- Validation: 5 tests (CH-2, CH-3, CH-4, CH-5, CH-6)
- Ordering: 1 test (CH-7)
- Typo guard: 1 test (CH-8)
- Contract: 1 test (CH-9)
- Answer reading: 5 tests (CA-1, CA-2, CA-3, CA-5, CA-8)
- Fallback: 1 test (CA-4)
- Timing: 1 test (CA-6)
- Addressing: 2 tests (CA-7, CH-10)

**Total:** 18 test functions

---

### CH-1: Help documents every option

- **Setup:** `HOME` set to an empty temporary directory
- **Command:** `clr chat help`
- **Expected behavior:** stdout mentions `--dir`, `--session`, `--timeout`, and `--raw`
- **Exit:** 0
- **Source:** [command/14_chat.md](../../../../docs/cli/command/14_chat.md)

---

### CH-2: A chat with nothing to say is rejected

- **Setup:** `HOME` set to an empty temporary directory
- **Command:** `clr chat`
- **Expected behavior:** stderr contains `needs a message` and points at `clr chat help`
- **Exit:** 1
- **Source:** [command/14_chat.md](../../../../docs/cli/command/14_chat.md)

---

### CH-3: Unknown option is rejected by name

- **Setup:** `HOME` set to an empty temporary directory
- **Command:** `clr chat hello --loudly`
- **Expected behavior:** stderr names `--loudly` and points at `clr chat help`
- **Exit:** 1
- **Source:** [command/14_chat.md](../../../../docs/cli/command/14_chat.md)

---

### CH-4: A non-numeric timeout is rejected, with the value quoted back

- **Setup:** `HOME` set to an empty temporary directory
- **Command:** `clr chat hello --timeout soon`
- **Expected behavior:** stderr contains `soon` — quoting the offending value back is what turns the message from a rule into a diagnosis
- **Exit:** 1
- **Source:** [command/14_chat.md](../../../../docs/cli/command/14_chat.md)

---

### CH-5: A flag at the end of the line, with no value, is rejected

- **Setup:** `HOME` set to an empty temporary directory
- **Command:** `clr chat hello --session`
- **Expected behavior:** stderr names `--session`
- **Rationale:** the alternative is silently ignoring it and talking to the wrong session
- **Exit:** 1
- **Source:** [command/14_chat.md](../../../../docs/cli/command/14_chat.md)

---

### CH-6: Two bare words mean the quotes were forgotten

- **Setup:** `HOME` set to an empty temporary directory
- **Command:** `clr chat say hello`
- **Expected behavior:** stderr suggests quoting the message
- **Rationale:** the failure this prevents is the expensive kind — chatting about only the first word succeeds, costs a model call, and answers a question nobody asked
- **Exit:** 1
- **Source:** [command/14_chat.md](../../../../docs/cli/command/14_chat.md)

---

### CH-7: An argument error starts no daemon

- **Setup:** `HOME` set to an empty temporary directory
- **Command:** `clr chat hello --loudly`
- **Expected behavior:** `$HOME/.claude/-daemon/daemon.sock` does not exist afterwards
- **Rationale:** the ordering guarantee that makes CH-2..CH-6 cheap. `chat` auto-starts a daemon, which is right when there is a chat to have and wrong on the way to reporting a typo
- **Exit:** 1
- **Source:** [command/14_chat.md](../../../../docs/cli/command/14_chat.md)

---

### CH-8: A near-miss subcommand is caught by the typo guard

- **Command:** `clr chatt`
- **Expected behavior:** stderr suggests `chat`
- **Rationale:** `chat` is in `KNOWN_SUBCOMMANDS`, so the Levenshtein-1 guard covers it; this is the test that says the registration was not forgotten
- **Exit:** 1
- **Source:** [command/14_chat.md](../../../../docs/cli/command/14_chat.md)

---

### CH-9: The onboarding hint's trigger phrase still matches the daemon's error

- **Setup:** none — the assertion is against `claude_daemon_core::Error::NoRegistration`'s own rendering
- **Expected behavior:** the rendered error contains `never registered a conversation id`
- **Rationale:** `chat` decides whether to print the first-run-prompt hint by matching that substring. Reword the daemon's error and the hint silently stops appearing — no compile error, no test failure anywhere else, just a worse message on the one path where a hint mattered most
- **Source:** [command/14_chat.md](../../../../docs/cli/command/14_chat.md)

---

### CH-10: `--session` addresses the daemon rather than a directory

- **Setup:** `HOME` set to an empty temporary directory; `clr daemon start` first; `DaemonGuard` stops it afterwards
- **Command:** `clr chat hello --session 00000000-0000-4000-8000-000000000000`
- **Expected behavior:** stderr contains `would not take the message` and does *not* contain `would not start`
- **Rationale:** the only case in this file that needs a real daemon, and it earns one. An ignored `--session` is invisible from outside: `chat` would fall through to matching on the working directory, find nothing, and *spawn* — a different failure with a different message, and on a machine with a working `claude` not a failure at all. Naming a session that cannot exist is what separates the two, because reaching the daemon fails at `send` and ignoring the flag fails at `spawn`
- **Exit:** 1
- **Source:** [command/14_chat.md](../../../../docs/cli/command/14_chat.md) — Algorithm — resolving the session, step 1

---

## Answer reading (`claude_storage_core/tests/transcript_answer_test.rs`)

These cases live one crate down, with the parser they exercise. They are documented here
because `clr chat` is the command whose promise they hold up.

Fixtures are hand-built JSONL, to the shape a real transcript has: the required-field
list comes from `claude_storage_core`'s own parser, the block mix (thinking, tool use,
tool result, text) from a recorded session. No process, no daemon, no model call — the
question is only what gets read out of a file that already exists.

Nothing there mutates `HOME` or any other process-global, so the file stays safe to run
concurrently with the rest of its crate's suite. CA-7 works with whatever `HOME` the
suite runs under, because its claim is the *shape* of the path and not where its root is.

---

### CA-1: An assistant text block past the mark is the answer

- **Setup:** a transcript with one user entry and one assistant entry whose only block is `text: "pineapple"`
- **Expected behavior:** `transcript_answer_since( path, 0, grace )` is `Some( "pineapple" )` — the user's own prompt is not in it
- **Source:** [command/14_chat.md](../../../../docs/cli/command/14_chat.md) — Algorithm — printing, step 3

---

### CA-2: Thinking and tool blocks are not the answer

- **Setup:** an assistant turn of a thinking block plus a `tool_use` block, a `tool_result` arriving as a *user* entry, then a second assistant turn with the text
- **Expected behavior:** the answer is exactly the final text; the thinking text, the tool name, and the tool-result id are all absent
- **Rationale:** the failure mode is not a crash. `claude_storage_core`'s own `content_text` flattens every block together, rendering tool calls as `Tool: {name} Input: {input:?}` and thinking as prose — reaching for the convenient accessor produces an answer that looks like an answer and is not one. The `tool_result`-as-user-entry line is why filtering on entry type alone would not have been enough
- **Source:** [command/14_chat.md](../../../../docs/cli/command/14_chat.md) — Algorithm — printing, step 3

---

### CA-3: The mark excludes the previous turn

- **Setup:** a two-entry transcript, `transcript_mark` taken, then a second turn appended
- **Expected behavior:** the mark is 2, and the answer is the second turn's text alone
- **Rationale:** this is the whole reason the mark is taken *before* `send` — without it every turn would reprint the entire conversation
- **Source:** [command/14_chat.md](../../../../docs/cli/command/14_chat.md) — Algorithm — printing, step 1

---

### CA-4: A missing transcript falls back rather than failing

- **Setup:** a path that was never written
- **Expected behavior:** `transcript_mark` is 0 and `transcript_answer_since` is `None`
- **Rationale:** a file that is not there is the ordinary case for the first chat in a directory, not an error. `None` is what routes `chat` to the terminal rendering and the accompanying stderr note
- **Source:** [command/14_chat.md](../../../../docs/cli/command/14_chat.md) — Algorithm — printing, step 4

---

### CA-5: Several text blocks are joined in order

- **Setup:** one assistant entry with two text blocks
- **Expected behavior:** both, in the order written, separated by a blank line
- **Source:** [command/14_chat.md](../../../../docs/cli/command/14_chat.md) — Algorithm — printing, step 3

---

### CA-6: The grace period waits for a transcript still being written

- **Setup:** a background thread writes the transcript 150ms after the wait begins; the wait is given 2 seconds
- **Expected behavior:** the answer is found
- **Rationale:** the turn ends when the session goes idle and quiet, which is slightly before Claude Code has finished flushing the file. Giving up at that instant would print a terminal dump for every successful chat. The margin is deliberately wide — the assertion is that waiting happens at all, not that it happens to a schedule
- **Source:** [command/14_chat.md](../../../../docs/cli/command/14_chat.md) — Algorithm — printing, step 4

---

### CA-7: `transcript_path` names the transcript after the session

- **Setup:** none; whatever `HOME` the suite runs under
- **Expected behavior:** the file name is `<session id>.jsonl` and the parent directory is the lossy encoding of the session's cwd (`/tmp/work` → `-tmp-work`)
- **Rationale:** the daemon holds a conversation id and nothing else. That the id alone names the file is the assumption the entire feature rests on
- **Source:** [command/14_chat.md](../../../../docs/cli/command/14_chat.md) — Notes, "Knowing *when* is not knowing *what*"

---

### CA-8: Non-conversation lines are neither counted nor printed

- **Setup:** a transcript interleaving `mode`, `summary`, `attachment`, and `system` lines with one real turn
- **Expected behavior:** the mark is 2 across six lines, and the answer is the assistant's text alone
- **Rationale:** all four appear in real transcripts and none is a conversation entry. A mark that counted *lines* would desynchronise every later turn — silently, and worse with each one
- **Source:** [command/14_chat.md](../../../../docs/cli/command/14_chat.md) — Algorithm — printing, step 1
