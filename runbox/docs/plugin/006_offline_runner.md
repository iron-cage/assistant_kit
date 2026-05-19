# Plugin: Offline runner

- **Status:** 🔧 Configurable — `dockerfile` param in `runbox.yml`; the dockerfile's `CMD` is the offline runner
- **Controls:** What command runs when `.test.offline` executes (no credentials, no network)
- **Mechanism:** `cmd_test_offline()` mounts the build-cache volume and runs the image with no command override — Docker uses the baked `CMD`; swapping the dockerfile swaps the offline runner

### Notes

`cmd_test_offline()` does not pass an explicit command to `docker run` — the image's baked `CMD` executes natively. The current dockerfile bakes `CMD cargo nextest run $CMD_SCOPE $CARGO_FEATURES --filter-expr "$CMD_FILTER"` from `runbox.yml` ARGs at build time. A different dockerfile defines its own `CMD`. Swapping plugin 003 (dockerfile) always swaps plugin 006 simultaneously.

### Example

For `workspace_test` built with `cmd_scope: --workspace`, `cargo_features: --all-features`, and `cmd_filter: "!test(lim_it) & !binary(behavior)"`, the dockerfile bakes:
```dockerfile
ARG CMD_SCOPE=--workspace
ARG CMD_FILTER=!test(lim_it) & !binary(behavior)
CMD cargo nextest run --workspace --all-features --filter-expr "!test(lim_it) & !binary(behavior)"
```
`cmd_test_offline()` runs:
```bash
docker run --rm -v workspace_test_target:/workspace/target workspace_test
# ↑ no command — Docker executes the baked CMD above
```
Setting `cargo_features: --no-default-features -F core_only` in `runbox.yml` and rebuilding bakes `--no-default-features -F core_only` into the CMD instead.
