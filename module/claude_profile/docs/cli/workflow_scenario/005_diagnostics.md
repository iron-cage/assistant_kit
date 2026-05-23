# Workflow Scenario :: 5. Diagnostics and Support

Collect environment information for troubleshooting.

```bash
# Show all file paths
clp .paths
# credentials:      /home/user/.claude/.credentials.json
# credential_store: /home/user/.persistent/claude/credential/
# projects:         /home/user/.claude/projects/
# stats:            /home/user/.claude/stats-cache.json
# settings:         /home/user/.claude/settings.json
# session-env:      /home/user/.claude/session-env/
# sessions:         /home/user/.claude/sessions/

# Check token state
clp .token.status
# valid — 2h47m remaining

# List all accounts with full metadata
clp .accounts

# Machine-readable snapshot for support tickets
clp .paths format::json > /tmp/diag-paths.json
clp .accounts format::json > /tmp/diag-accounts.json
clp .token.status format::json > /tmp/diag-token.json
```

**When to use:** Filing support tickets, debugging environment issues, verifying correct `~/.claude/` layout.
