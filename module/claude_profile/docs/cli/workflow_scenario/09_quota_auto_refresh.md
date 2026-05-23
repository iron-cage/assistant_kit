# Workflow Scenario :: 9. Quota Fetch with Auto Token Refresh

Use `refresh::1` to silently refresh expired tokens so every account shows current quota data rather than auth error rows.

```bash
# Without refresh::1 — expired accounts show error rows
clp .usage
#   Account          Expires   5h Left  ...
# ✓ i12@wbox.pro    in 7h     86%      ...
#   i6@wbox.pro     EXPIRED   —        (auth error: 401)

# With refresh::1 — expired tokens silently refreshed before the fetch
clp .usage refresh::1
#   Account          Expires     5h Left  ...
# ✓ i12@wbox.pro    in 7h 24m  86%      ...
#   i6@wbox.pro     in 5h 02m  100%     ...
# (i6's token was refreshed in-place; credential file updated on disk)

# Combine with live mode for sessions where tokens may expire mid-session
clp .usage live::1 refresh::1 interval::60

# JSON output is also supported; refresh is invisible in JSON output
clp .usage refresh::1 format::json
# [
#   {"account":"i12@wbox.pro","session_5h_left_pct":86,...},
#   {"account":"i6@wbox.pro","session_5h_left_pct":100,...}
# ]
```

**When to use:** When accounts have expired tokens and you want quota data for all of them without manually triggering a re-login. The credential file is updated on disk so subsequent `.usage` calls (without `refresh::1`) also use the fresh token.

**Note:** When `refresh::1` silently fails (trace shows `run_isolated: OK credentials=None`), the `refreshToken` itself is expired — use [`.account.relogin`](../command/account.md#command--12-accountrelogin) for full browser re-authentication.
