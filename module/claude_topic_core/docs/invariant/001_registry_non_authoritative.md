# Invariant: Registry Non-Authoritative

### Scope

- **Purpose**: Fix the authority relationship between the fork-topic name registry and the session files it names, so no consumer treats a listing as proof a conversation exists.
- **Governs**: `src/registry.rs`, and every consumer of `registry::list`.
- **In Scope**: What a registry entry does and does not assert; failure behaviour on write.
- **Out of Scope**: The merged listing built on top of it (→ [feature/002](../feature/002_topic_enumeration.md)); Claude Code's own session storage.

### Rule

**A registry entry asserts that a name was once recorded. It asserts nothing about
whether a session exists now.** The session file is the authority; the registry is
an index that exists only because `UUIDv5` is one-way.

Three consequences, all enforced:

1. **`record` is warn-never-fatal.** A failed index write prints a warning and
   returns. The run that triggered it already succeeded or failed on its own terms,
   and a listing-index write must never change that verdict.
2. **`list` tolerates absence.** A missing file, an unreadable file, or an
   unencodable base yields an empty list. No fork topics recorded is an ordinary
   state, not an error.
3. **Existence is re-derived, never assumed.** `enumerate` resolves each listed name
   back through the shared `UUIDv5` rule and stats the resulting file. An entry
   whose session file was deleted lists with `sessions == 0`, and
   `enumerate_live` drops it.

The corollary is the reason this is written down: **a consumer must never treat
`registry::list` output as a set of addressable topics.** It is a set of names that
were recorded. `enumerate_live` is the addressable set.

**Rationale — why an index at all.** A fork topic's identity is
`UUIDv5( canonical base, name )`. That is one-way by construction, which is what makes
it deterministic and collision-free — and also what makes the name unrecoverable from
the file. Without a side channel, fork topics would be perfectly usable when named
explicitly and completely invisible to any listing. Every alternative (embedding the
name in the transcript, a manifest inside the session dir) either couples this crate
to Claude Code's file format or puts writes on the path of the run itself.

**Rationale — why not authoritative.** Making the registry authoritative would mean
keeping it consistent with a directory this crate does not own and does not write.
Claude Code creates session files; a user can delete them. Any index that claims to
mirror that faithfully is claiming a guarantee it cannot deliver, and the failure
mode — listing a topic that is not there — is worse than the honest one, which is
listing it with zero sessions.

### Verification

The mechanical check is `tests/registry_test.rs` plus `ten08` in
`tests/enumerate_test.rs`, which asserts that a registry entry with no session file
still lists, with `sessions == 0`, and is excluded from `enumerate_live`:

```bash
cd module/claude_topic_core && ./verb/test
```

By hand, the two halves can be compared directly — the registry file is plain text:

```bash
cat "${CLR_TOPIC_REGISTRY_DIR:-$HOME/.clr/topics}"/*   # names ever recorded
clr topics                                             # names with a conversation
```

The first list is always a superset of the second's fork rows. A name in the first
and not the second is exactly the case this invariant exists to make safe.

To confirm the write path never escalates a failure, point the registry somewhere
unwritable and check the exit code:

```bash
CLR_TOPIC_REGISTRY_DIR=/proc/nonexistent clr topic --topic scratch "hello"
echo "exit: $?"   # governed by the run, not by the failed index write
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/registry.rs` | `record`, `list`, and the warn-never-fatal policy |
| source | `src/enumerate.rs` | Re-deriving existence from the session file |
| doc | [feature/002_topic_enumeration.md](../feature/002_topic_enumeration.md) | The merged listing and its live filter |
| test | `tests/registry_test.rs` | Append-if-missing, newline refusal, absence tolerance |
| test | `tests/enumerate_test.rs` | ten08 — an entry with no file lists with zero sessions |
