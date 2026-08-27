# ENVELOPE CLASS: Detached

### Scope

- **Purpose**: Define Class C — the five top-level kinds carrying none of the nine common fields, not even `sessionId`, and the kind-specific handles that are their only means of correlation.
- **Responsibility**: Authoritative instance for the Class C field contract, its membership, its correlation handles, and the attribution loss that follows from having no `sessionId`.
- **In Scope**: Membership, the guaranteed-absent field set, the per-kind correlation handle table, and the parsing consequences of full detachment.
- **Out of Scope**: Per-kind payload semantics (→ [`../envelope/`](../envelope/readme.md)); the other two classes (→ [001](001_full_envelope.md), [002](002_session_scoped.md)).

### Membership

Five kinds, 8,912 lines — 0.18% of the store:

| Kind | Envelope Instance | Lines |
|------|-------------------|------:|
| `file-history-snapshot` | [012_file_history_snapshot.md](../envelope/012_file_history_snapshot.md) | 8,016 |
| `started` | [015_started.md](../envelope/015_started.md) | 329 |
| `result` | [016_result.md](../envelope/016_result.md) | 285 |
| `summary` | [017_summary.md](../envelope/017_summary.md) | 178 |
| `fork-context-ref` | [018_fork_context_ref.md](../envelope/018_fork_context_ref.md) | 104 |

### Field Contract

**Guaranteed absent** on 100% of Class C lines — all nine: `uuid`, `parentUuid`, `timestamp`, `sessionId`, `cwd`, `version`, `gitBranch`, `userType`, `isSidechain`.

A Class C line consists of its `type` discriminator and its payload. Nothing else.

### Correlation

Each kind carries its own handle instead of a common field. Every handle listed is present on 100% of that kind's lines:

| Kind | Handle | Resolves to |
|------|--------|-------------|
| `file-history-snapshot` | `messageId` | A Class A entry's `uuid` |
| `summary` | `leafUuid` | The Class A thread leaf being summarized |
| `started`, `result` | `agentId` + `key` | A subagent invocation, cache-keyed |
| `fork-context-ref` | `parentSessionId` + `parentLastUuid` | The forked-from session and its last entry |

**Session attribution has exactly two sources**, and both are external to the line:

1. **The file the line was found in.** Session files are named by session ID, so the path supplies what the line does not.
2. **Resolving the handle against Class A entries**, which do carry `sessionId`.

**A consumer that concatenates or re-shards session files loses Class C attribution irrecoverably** unless it resolves handles first. This is the one class where reading a line out of its file context destroys information that cannot be recovered from the line itself. `fork-context-ref` is the partial exception: its `parentSessionId` names a session, but that is the *parent* session, not the one the line belongs to.

### Notes

**`started` and `result` pair, and the pair does not balance.** 329 initiations against 285 completions — 44 subagent invocations began without a recorded result. Because both kinds carry `agentId` and `key`, the gap is directly enumerable rather than merely countable, which makes this the most tractable subagent-failure signal in the log.

**`key` is a cache key, not an identifier.** Two invocations with identical inputs share a `key`, so `key` alone does not uniquely identify an invocation; the pair `(key, agentId)` does. A consumer joining on `key` alone will conflate cache-equivalent invocations.

**`fork-context-ref` is the documented exception to no-cross-session-links.** [`018_b18_no_cross_session_links.md`](../behavior/018_b18_no_cross_session_links.md) establishes that entries do not reference other sessions. This kind does — at session rather than entry granularity, which is why the behavior rule and this exception coexist rather than conflict.

**`summary` is named in the storage invariant.** Like `queue-operation`, it is one of the four non-conversation types [`003_entry_type_format.md`](../../../../module/claude_storage/docs/invariant/003_entry_type_format.md) enumerates, and its fixture-versus-production `uuid` discrepancy is corrected in [readme.md](readme.md).

**Smallest class, largest parsing hazard.** At 0.18% of the store, Class C is easy to miss when validating a parser against sampled data — and it is the class whose lines break every assumption a consumer might reasonably hold about a session log line.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| envelope_class | [readme.md](readme.md) | Class master index, presence matrix, and the `uuid` correction |
| envelope_class | [001_full_envelope.md](001_full_envelope.md) | Class A — the class every handle here resolves against |
| envelope_class | [002_session_scoped.md](002_session_scoped.md) | Class B — `sessionId` and payload only |
| envelope | [`../envelope/readme.md`](../envelope/readme.md) | All 19 top-level kinds this class partitions |
| behavior | [`../behavior/018_b18_no_cross_session_links.md`](../behavior/018_b18_no_cross_session_links.md) | No-cross-session-links rule that `fork-context-ref` excepts |
| behavior | [`../behavior/021_b21_fork_session.md`](../behavior/021_b21_fork_session.md) | Fork behavior producing `fork-context-ref` |
| behavior | [`../behavior/037_b37_subagent_cache_ttl.md`](../behavior/037_b37_subagent_cache_ttl.md) | Subagent cache isolation underlying `key` |
| storage | [`../storage/001_projects_directory.md`](../storage/001_projects_directory.md) | File naming that supplies session attribution these lines lack |
| invariant | [`../../../../module/claude_storage/docs/invariant/003_entry_type_format.md`](../../../../module/claude_storage/docs/invariant/003_entry_type_format.md) | Skip-handling contract naming `summary` |
