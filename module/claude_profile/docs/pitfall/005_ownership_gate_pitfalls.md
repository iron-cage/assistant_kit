# Pitfall: Ownership Gate Pitfalls

### Pattern

Ownership gates must guard BOTH `is_owned=false` AND `is_occupied_elsewhere=true`. Missing either half allows operations that damage active sessions on other machines.

### Pitfall 1 — Refresh gate missing occupied-elsewhere guard (BUG-303)

`should_refresh()` in `refresh_predicate.rs` checked `!aq.is_owned` only. An account owned by this machine but actively in use on another machine (`is_occupied_elsewhere=true`) was still submitted to `refresh_account_token()`. The refresh overwrote `accessToken`/`refreshToken` on disk, immediately invalidating the live session on the other machine.

**Fix (G2 gate):** `!aq.is_owned || aq.is_occupied_elsewhere` — both conditions cause skip.

### Pitfall 2 — Touch gate missing occupied-elsewhere guard (BUG-302)

Same pattern as BUG-303 in `apply_touch()`. The G4 gate only checked `!aq.is_owned`. Touching an occupied account spawns a subprocess that competes with the remote session.

**Fix (G4 gate):** `!aq.is_owned || aq.is_occupied_elsewhere` — both cause skip.

### Pitfall 3 — Fetch gate G1b missing occupied-elsewhere path (BUG-305)

`fetch.rs` had the solo gate and the `!is_owned` path, but no explicit occupied-elsewhere path. An occupied account with `is_owned=true` proceeded to live `fetch_oauth_usage()` — potentially competing with the remote session's HTTP traffic and using quota tokens.

**Fix (G1b gate):** After the solo gate: `if !is_current && occupied_elsewhere.contains(&acct.name) → approximate_quota()`.

### Pitfall 4 — `reason_label()` missing occupied-elsewhere trace (BUG-306)

`refresh.rs` trace reason computation had `reason: ok` as the fallback for owned+non-cached+non-401/403 accounts. An occupied-elsewhere account with a valid cached result hit this fallback and logged `reason: ok` — misleading trace output.

**Fix:** Extract `reason_label()` function with explicit `is_occupied_elsewhere` branch before the fallback.

### General Rule

Every gate that protects against cross-machine interference MUST check BOTH:
1. `!aq.is_owned` (this machine doesn't own the account)
2. `aq.is_occupied_elsewhere` (another machine is actively using the account)

These are independent conditions. An owned account can be occupied elsewhere; a non-owned account may or may not be occupied. Both must be guarded independently.

### Cross-References

| File | Relationship |
|------|-------------|
| [feature/036_account_ownership.md](../feature/036_account_ownership.md) | Ownership gates G1–G8 |
| [feature/017_token_refresh.md](../feature/017_token_refresh.md) | G2 gate (refresh) |
| [feature/024_session_touch.md](../feature/024_session_touch.md) | G4 gate (touch) |
| [feature/061_solo_token_conservation.md](../feature/061_solo_token_conservation.md) | G1b gate (fetch) |
| [state_machine/004](../state_machine/004_ownership_lifecycle.md) | Ownership lifecycle |
