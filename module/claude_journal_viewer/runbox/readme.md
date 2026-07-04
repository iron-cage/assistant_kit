# runbox/

Container operation files for `claude_journal_viewer`.

| File | Responsibility |
|------|----------------|
| `plugins.sh` | NEXTEST_FILTER hook: redirects TEST_SCRIPT to `test_only.d/l1` for targeted runs. |
| `runbox` | Wrapper: auto-discovers and delegates to `runbox-run` with `runbox.yml`. |
| `runbox.yml` | Configuration: image, test/lint scripts, plugins, and build contexts. |
