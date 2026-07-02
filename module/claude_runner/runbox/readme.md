# runbox/

Container operation files for `claude_runner`.

| File | Responsibility |
|------|----------------|
| `runbox` | Wrapper: auto-discovers and delegates to `runbox-run` with `runbox.yml`. |
| `runbox.yml` | Configuration: image, test/lint/run scripts, plugins, and build contexts. |
| `plugins.sh` | Project plugin: forwards `NEXTEST_FILTER` into container for `verb/test1`. |
