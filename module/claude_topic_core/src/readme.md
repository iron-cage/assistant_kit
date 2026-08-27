# src/

Core library implementation for `claude_topic_core`.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `lib.rs` | Crate root, module wiring, public re-exports |
| `identity.rs` | What a topic name resolves to: base, mechanism, session file |
| `registry.rs` | The side-channel index that makes fork-mode names listable |
| `enumerate.rs` | Which topics exist under a base, both mechanisms merged |
| `select.rs` | Which topic to hand a prompt to, idle-first |
| `pool.rs` | Naming anonymous topics idempotently |
| `lock.rs` | Advisory per-topic exclusion with dead-owner reclaim |
