# Command :: 13. `.account.rotate` — Integration Tests

> **DEPRECATED** — `.account.rotate` is now a redirector (Feature 016). Always exits 1 with migration message. Rotation moved to `.usage rotate::1` (Feature 038). Test cases below verify redirector behavior only.

| # | Test | Conditions | Exit |
|---|------|-----------|------|
| IT-1 | Always exits 1 | Any invocation of `.account.rotate` | 1 |
| IT-2 | Error message references `.usage rotate` | No args; stderr/stdout contains `.usage rotate` | 1 |
| IT-3 | No mutation on exit 1 | Two accounts present; `_active` unchanged after deprecated call | 1 |
