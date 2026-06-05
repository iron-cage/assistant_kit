# Workflow Scenario :: 8. Live Quota Monitoring Dashboard

Run `.usage` as a continuous ambient display that auto-refreshes without re-invoking the command.

```bash
# Start the live monitor (default: refresh every 30 seconds)
clp .usage live::1
# Quota
#
#   ●  Account              5h Left     5h Reset    7d Left  7d(Son)  7d Reset   Expires     ~Renews      → Next
# → 🟢 bob@example.com      🟢 100%    in 4h 58m  🟢 88%   28%      in 6d 14h  in 5h 02m   ~in 30d      in 6d 14h +7d
# ✓ 🟢 alice@example.com    🟢 86%     in 3h 19m  🟢 65%   35%      in 4d 23h  in 7h 24m   ~in 6d       in 4d 23h +7d
#   🔴 dave@example.com     —          —           —        —        —          EXPIRED      ?            —
#
# Valid: 2 / 3   ->  Next by strategy:
#   endurance  bob@example.com     100% session, 5h resets in 4h 58m
#   drain      bob@example.com     28% 7d left, 7d resets in 6d 14h
#
#   Next update in 0:29 (at 14:32:07 UTC)  [Ctrl-C to exit]

# Slower refresh with jitter to spread out API calls across time
clp .usage live::1 interval::120 jitter::15

# Combine with auto token refresh for long-running sessions
clp .usage live::1 refresh::1 interval::60

# Incompatible: live mode with JSON output exits 1 before any fetch
clp .usage live::1 format::json
# error: live monitor mode is incompatible with format::json
```

**When to use:** Long-running work sessions where you want an always-visible quota dashboard in a side terminal. Set `interval::` to 120+ and `jitter::` to 10–30 to reduce API call frequency over many hours.
