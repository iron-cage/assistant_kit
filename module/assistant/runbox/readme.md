# runbox/

Container configuration for `assistant` test execution.

| File | Responsibility |
|------|----------------|
| `plugins.sh` | NEXTEST_FILTER hook: redirects TEST_SCRIPT to `test_only.d/l1` for targeted runs. |
| `runbox` | Entry script dispatching `.test` and other verbs inside container |
| `runbox.yml` | Container image, user, cmd_scope, and plugin mount declarations |
