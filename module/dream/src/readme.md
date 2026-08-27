# src/

Feature-gated facade re-exporting coding agent `*_core` crates.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `lib.rs` | Feature-gated `pub use` re-export modules — one per `*_core` crate |

### Scope

**In Scope:**
- Feature-gated re-export wiring (`common`, `storage`, `profile`, `runner`, `version`, `assets`, `quota`)
- Aggregating the `*_core` crates into one dependency for consumers that only need core (non-CLI) surfaces

**Out of Scope:**
- Any actual implementation — all behavior lives in the re-exported `*_core` crates; see `docs/invariant/001_no_own_logic.md`
- Full-featured (CLI-surface) re-exports (→ `assistant_kit` crate)

### Invariants

Formally documented — see `docs/invariant/readme.md`:
- [`001_no_own_logic.md`](../docs/invariant/001_no_own_logic.md) — this crate re-exports only; it must never contain its own logic.
