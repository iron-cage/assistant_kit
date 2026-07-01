# Pitfall: Ownership Gate Pitfalls

### Scope

- **Purpose**: Document failure modes in ownership and occupied-elsewhere gate enforcement.
- **Responsibility**: Covers missing `is_occupied_elsewhere` guards in refresh, touch, fetch, and trace code; scope of explicit command gates.
- **In Scope**: G1b, G2, G4 gate pitfalls; BUG-303, BUG-302, BUG-305, BUG-306; explicit command gate scope.
- **Out of Scope**: Ownership lifecycle states (→ state_machine/004); ownership feature parameters (→ feature/036).

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

### Pitfall 5 — Occupied-elsewhere accounts bypass explicit command gates

G5 (`.account.use`), G6 (`.account.delete`), and G7 (`.account.relogin`) check `!is_owned` only. An occupied-elsewhere account has `is_owned=true` and therefore passes all three gates without restriction.

This is intentional by design: occupied-elsewhere accounts are owned by this machine, so explicit manipulation is permitted. Protection for occupied-elsewhere accounts exists only in the automatic pipeline (G1b blocks HTTP fetch; G2 blocks refresh; G4 blocks touch) and in `find_first_eligible` Gate 3 (blocks auto-switch selection). Explicit command gates do not participate.

**Contrast:** a non-owned account (`is_owned=false`) is blocked by G5/G6/G7/G8 for all mutation commands. An occupied-elsewhere account (`is_owned=true`, `is_occupied_elsewhere=true`) is blocked only by the automatic pipeline and Gate 3 — never by the explicit command gates.

**Pitfall for system extension:** when adding a new mutation command, do not assume the existing ownership gate pattern (`!is_owned`) is sufficient to protect occupied-elsewhere accounts. If the new command must not disturb a remote session, add an explicit `is_occupied_elsewhere` check independently.

### General Rule

Every gate that protects against cross-machine interference MUST check BOTH:
1. `!aq.is_owned` (this machine doesn't own the account)
2. `aq.is_occupied_elsewhere` (another machine is actively using the account)

These are independent conditions. An owned account can be occupied elsewhere; a non-owned account may or may not be occupied. Both must be guarded independently.

### Features

| File | Relationship |
|------|-------------|
| [feature/036_account_ownership.md](../feature/036_account_ownership.md) | Ownership gates G1–G8 |
| [feature/017_token_refresh.md](../feature/017_token_refresh.md) | G2 gate (refresh) |
| [feature/024_session_touch.md](../feature/024_session_touch.md) | G4 gate (touch) |
| [feature/061_solo_token_conservation.md](../feature/061_solo_token_conservation.md) | G1b gate (fetch) |

### State Machines

| File | Relationship |
|------|-------------|
| [state_machine/004](../state_machine/004_ownership_lifecycle.md) | Ownership lifecycle |
