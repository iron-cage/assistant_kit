# Decision: Unknown Flags Rejected

**ID:** D5 · **Category:** Parsing · **Status:** ✅ Adopted

### Scope

- **Purpose**: Record why the parser validates against an explicit whitelist instead of forwarding unrecognized flags to `claude`.
- **Responsibility**: Rationale for whitelist validation and for erroring rather than passing through.
- **In Scope**: The whitelist rule, the `--help` hint on failure, and the two failure modes it prevents.
- **Out of Scope**: The whitelist's actual contents (→ [`../cli/param/readme.md`](../cli/param/readme.md)); parser construction (→ [007_hand_rolled_parser.md](007_hand_rolled_parser.md)).

### Decision

An explicit whitelist of known flags. Unknown flags produce an error with a `--help` hint.

### Rationale

Two failure modes, both silent under a passthrough design:

1. **Typos** — `--modle sonnet` forwarded verbatim is either ignored or misinterpreted downstream, and the user sees a result produced under the wrong settings with no indication why.
2. **Accidental passthrough to claude** — a flag `clr` does not know about reaching the `claude` binary means `clr`'s own contract no longer describes what ran.

Rejecting is recoverable in one keystroke; a silently wrong run is not recoverable at all, because nothing signals that it happened.

### Consequence

Every flag `clr` accepts is enumerated somewhere it can be listed, tested, and documented. Adding a claude-native flag to the surface is a deliberate act rather than a side effect.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Decision collection index |
| decision | [007_hand_rolled_parser.md](007_hand_rolled_parser.md) | The parser that enforces this whitelist |
| cli | [`../cli/param/readme.md`](../cli/param/readme.md) | The parameter surface the whitelist is drawn from |
| test | `../../tests/cli_args_test.rs` | Whitelist rejection coverage |
