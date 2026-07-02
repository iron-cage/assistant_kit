# run

Shell scripts for `claude_profile` container operations.

| File | Responsibility |
|------|----------------|
| `verb-run` | Universal verb dispatcher: resolves flat-file or directory-form verbs by VERB_LAYER. |
| `plugins.sh` | Project plugin: forwards `NEXTEST_FILTER` into container for `verb/test1`. |
