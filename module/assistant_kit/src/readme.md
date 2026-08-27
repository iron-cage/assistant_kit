# src/

Layer 3 library facade re-exporting all Layer 2 full-featured crates behind feature flags.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `lib.rs` | Feature-gated `pub use` re-export modules — one per Layer 2 crate |

### Scope

**In Scope:**
- Feature-gated re-export wiring (`profile`, `runner`, `version`, `assets`, `storage`, `full`, `enabled`)
- Giving library consumers access to the complete CLI command surface without depending on a binary

**Out of Scope:**
- Any actual implementation — all behavior lives in the re-exported Layer 2 crates (`claude_profile`, `claude_runner`, `claude_version`, `claude_assets`, `claude_storage`)
- `*_core`-only re-exports (→ `dream` crate)
