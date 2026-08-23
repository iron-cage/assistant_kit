# Algorithm: Session Liveness

### Scope

- **Purpose**: Determine which projects have a Claude Code process attached, and which of their conversations that process is driving.
- **Responsibility**: The two inference signals, their combination rule, the fallback when one is absent, and the reporting constraint that follows from neither signal being able to prove a negative.
- **In Scope**: Process-table probing, `history.jsonl` correlation, project-versus-session granularity, the working/waiting split, detection availability.
- **Out of Scope**: Rendering of the resulting state (→ `../cli/command/07_projects.md`), the `live::` filter's own contract (→ `../cli/param/44_live.md`), path encoding (→ `../invariant/001_path_encoding.md`).

### Abstract

Claude Code stores no liveness marker. It writes no per-session lock file, does not hold the session JSONL open between appends, and does not carry the session id in its process environment — all three were checked directly. Liveness is therefore **inferred**, never read.

The naive inference — "written recently means running" — is not merely imprecise, it is wrong in both directions:

| Measurement (914-project store, 38 attached processes) | Value |
|---|---|
| Median idle time of an attached session | 108 s |
| Longest idle time of an attached session | 3336 s (56 min) |
| Attached sessions idle > 5 min | 11 of 38 (29%) |
| Attached sessions idle > 15 min | 8 of 38 (21%) |

A five-minute recency cutoff would call 29% of genuinely live sessions dead, and would call every session that ended four minutes ago alive. **Recency and liveness are orthogonal facts and are computed and displayed separately** — recency stays in the `LAST` column, liveness becomes `STATUS`.

### Signals

**Signal 1 — attached processes.** The main `claude` process keeps its working directory at the session's project root for the session's whole life (the Bash tool's own `cd` calls happen in subprocesses and never move it). So for every entry in `/proc` whose `comm` is exactly `claude`, `/proc/<pid>/cwd` names a live project.

- Matching `comm` rather than the command line is deliberate: a command-line scan also collects wrapper scripts, `grep claude`, and the probing process itself.
- Measured resolution rate: 38 of 39 distinct live cwds mapped to an existing project directory. The one miss was a session that had not yet written anything.
- Granularity is **project-level**. This signal is authoritative for *whether* a project is live and silent about *which* of its conversations.

**Signal 2 — `~/.claude/history.jsonl`.** One record per submitted prompt:

```json
{ "display" : "…", "project" : "/home/alice/pro/app", "sessionId" : "98da5af5-…", "timestamp" : "1787434680819" }
```

`project` is the **unencoded** path, so it does not inherit the ambiguity of [path encoding](../invariant/001_path_encoding.md), and `sessionId` names the exact conversation. The newest record for a project therefore identifies the session receiving input — which is what raises the answer from project-level to session-level.

- Cross-checked against mtime ranking over every attached cwd: 12 agreed, 0 disagreed, 2 had no record at all.
- The two absentees were headless (`--print`) sessions, which take their prompt on argv and never write history.
- The file grows without bound, so only its tail (512 KiB) is read, and only for projects already known to be attached.

### Algorithm

**Keying.** Both signals produce absolute filesystem paths; rows produce a display path decoded out of a storage directory name. That decode is lossy — `_` and `/` both encode to `-` — so a decoded path is a guess. Both sides are put through the *identical* encode-then-decode round trip:

```
key( path ) = decode_project_display( encode_path( path ) )
```

Because `encode(decode(x)) == x` for any storage directory name, both sides land on the same guess and always match, even where the guess is wrong.

**Procedure.**

1. Walk the process table. For each numeric entry whose `comm` is `claude`, read `cwd` and increment `attached[ key( cwd ) ]`.
2. If nothing is attached, stop — there is nothing to report and nothing to correlate.
3. Read the tail of `history.jsonl` newest-first. For each record whose project key is attached, append its `sessionId` to `driving[ key ]` unless already present, capped at that project's attached count. Stop once every attached slot is filled.
4. A **project** is live iff `attached[ key ] > 0`.
5. A **conversation** is live iff its project is attached *and* either
   - `driving[ key ]` is non-empty and contains its session id — history is authoritative whenever it has anything to say, precisely because the newest session by mtime is frequently not the live one; or
   - `driving[ key ]` is empty and the conversation's mtime rank is below the attached count — the headless fallback, where mtime order is the only remaining signal.
6. Split each live row by recency: written within 60 s ⇒ **working** (mid-turn), otherwise **waiting** (a terminal is open and idle). Recency is consulted *only* at this step, never as a liveness test of its own.

### Constraints

**Detection can only report positives.** The process table is Linux-only, and inside a container it lists the container's processes rather than the host's. In both cases every project reads as unattached, which is indistinguishable from a store where nothing is running. Two consequences are binding on every consumer:

- The `STATUS` column is rendered only when at least one attached process was actually found. A blank column means "nothing detected" and must never be presented as "nothing live".
- [`live::1`](../cli/param/44_live.md) reports the ambiguity explicitly rather than returning a silently empty list.

**Agent sidecars are never marked.** `history.jsonl` only ever names root session ids, so an agent could only be marked through the mtime-rank fallback — where it would be a coincidence of ordering rather than evidence.

**Session ids are not globally unique.** The same session id is observed under several project directories with differing entry counts, so every lookup is keyed by `(project, session id)` and never by session id alone.

**A project running both a headless and an interactive session marks only the interactive one.** Step 5 prefers history whenever it is non-empty, and the headless session is absent from it. Accepted: the alternative — unioning history with mtime rank — reintroduces exactly the false positive step 5 exists to prevent.

### Verification

```bash
# Attached processes and the projects they name
for p in $( pgrep -x claude ); do readlink /proc/$p/cwd; done | sort

# Which session id each attached project is driving
tail -500 ~/.claude/history.jsonl | jq -r '[ .project, .sessionId ] | @tsv' | tac | sort -u -k1,1

# What the algorithm concludes
clg .projects scope::global live::1 detail::sessions
```

### Referenced Documents

| Document | Relationship |
|----------|--------------|
| [`../invariant/001_path_encoding.md`](../invariant/001_path_encoding.md) | Supplies the lossy encoding the keying rule round-trips through |
| [`../cli/command/07_projects.md`](../cli/command/07_projects.md) | Renders the resulting state as the `STATUS` column and session tags |
| [`../cli/param/44_live.md`](../cli/param/44_live.md) | Exposes the project-level verdict as a filter |
