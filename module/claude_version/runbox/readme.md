# runbox

Container runner integration scripts and config for `claude_version`.

| File | Responsibility |
|------|----------------|
| `runbox` | Entry point script for container-based test execution |
| `runbox.yml` | Container runner configuration |
| `plugins.sh` | NEXTEST_FILTER hook; redirects to `test_only.d/l1` for targeted runs |
