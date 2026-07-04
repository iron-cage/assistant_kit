# run

Shell scripts for `claude_profile` container operations.

| File | Responsibility |
|------|----------------|
| `verb-run` | Universal verb dispatcher: resolves flat-file or directory-form verbs by VERB_LAYER. |
| `plugins.sh` | NEXTEST_FILTER hook: redirects TEST_SCRIPT to `test_only.d/l1` for targeted runs. |
