# runbox/

| File | Responsibility |
|------|----------------|
| `plugins.sh` | NEXTEST_FILTER hook: redirects TEST_SCRIPT to `test_only.d/l1` for targeted runs. |
| `runbox` | Walk-up discovery wrapper script (entry point for container runner) |
| `runbox.yml` | Container runner config: image, test script, workspace settings |
