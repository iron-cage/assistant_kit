# Workflow Scenario :: 10. Account Re-Authentication Recovery

**Objective:** Restore quota access for an account whose `refreshToken` is expired by triggering full browser re-authentication via `.account.relogin`.

**Complexity:** Low
**Duration:** 2–5 minutes plus interactive browser login time
**Prerequisites:** Account exists in credential store; `claude` binary on `PATH`; interactive TTY (not piped or CI)

### Workflow Steps

**Step 1: Confirm the Refresh Token is Dead**

```bash
# Run usage with trace to identify accounts where silent refresh fails
clp .usage refresh::1 trace::1
# [trace] refresh carol@example.com  switch_account: OK
# [trace] refresh carol@example.com  run_isolated: OK credentials=None   <- dead refreshToken
#
#   Account          Expires   Sub  ~Renews  5h Left
# ✓ alice@example.com    in 7h     86%
#   carol@example.com     EXPIRED   —       (auth error: 401)
#
# credentials=None means refresh::1 cannot renew the token — browser login required.
```

**Step 2: Re-Authenticate via Browser Login**

```bash
# Must run in an interactive TTY — Claude opens a browser or in-terminal login prompt
clp .account.relogin name::carol@example.com
# [relogin] switching to carol@example.com...
# [relogin] spawning claude for browser re-authentication (Ctrl-C to abort)
#   ... (complete the login flow in the browser or terminal) ...
# [relogin] credentials updated — saving as carol@example.com
# [relogin] restored active account: alice@example.com
# relogin successful
```

**Step 3: Verify Recovery**

```bash
clp .usage
#   Account          Expires     Sub  ~Renews  5h Left
# ✓ alice@example.com    in 7h 18m  84%
#   carol@example.com     in 7h 02m  100%    <- restored with fresh token
```

### Error Handling

**Login abandoned — claude exits without updating credentials (exit 3):**

```bash
# Ctrl-C during login or session timeout causes exit 3.
# The active account is still restored. Retry when ready.
clp .account.relogin name::carol@example.com
```

**Non-TTY environment (piped shell, CI pipeline):**

```bash
# .account.relogin requires an inherited TTY — run from an interactive terminal.
# Preview the steps without executing:
clp .account.relogin name::carol@example.com dry::1
# [dry-run] would re-authenticate 'carol@example.com' via browser login
```

**Account not found (exit 2):**

```bash
# Account must be saved in the credential store first.
clp .account.save name::carol@example.com
clp .account.relogin name::carol@example.com
```

### Workflow Variations

**Prefix shorthand:**

```bash
# Unique prefix resolves to the matching account
clp .account.relogin i3
```

**Multiple dead accounts — run sequentially in an interactive terminal:**

```bash
# Identify all accounts with dead refresh tokens
clp .usage refresh::1 trace::1 2>&1 | grep "credentials=None"

# Re-authenticate each in turn (each requires a separate browser login session)
clp .account.relogin name::carol@example.com
clp .account.relogin name::bob@example.com
```

**When to use:** After `clp .usage refresh::1` or `refresh::1 trace::1` shows `credentials=None` — the silent subprocess refresh failed, indicating the `refreshToken` itself is expired. See [workflow 9](009_quota_auto_refresh.md) for the automatic refresh path.
