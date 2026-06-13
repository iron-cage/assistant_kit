# Plugin: `bin_plugin`

- **Status:** ✅ Configured — defined in `runbox/plugins.sh`; configured via `runbox.yml`
- **Controls:** Host binary injected into container with a working volume
- **Mechanism:** `which name` on host → bind-mount binary `:ro` into container; named Docker volume for working directory; implemented in `_plugin_test_args` / `_plugin_shell_extra_args` in `runbox/plugins.sh`

### Notes

Configured via `runbox.yml` key `bin_plugin: name:/container/path`. Current use: `w3:/usr/local/bin/w3`. Plugin logic lives entirely in `runbox/plugins.sh` — core `runbox-run` has no plugin knowledge. A second binary plugin slot requires additions to `plugins.sh` only; `runbox-run` is unchanged.

### Example

```yaml
bin_plugin: w3:/usr/local/bin/w3
bin_plugin_volume: /tmp/will_test_targets
```
`cmd_test()` resolves `which w3` on the host, then builds:
```bash
-v workspace_test_plugin_targets:/tmp/will_test_targets
-v /usr/local/bin/w3:/usr/local/bin/w3:ro
-e CARGO_TARGET_DIR=/tmp/will_test_targets
```
Inside the container `w3` is callable at `/usr/local/bin/w3` exactly as on the host. The `workspace_test_plugin_targets` volume persists `w3`'s compilation artifacts across `.test` invocations — repeated calls reuse prior builds instead of recompiling. `CARGO_TARGET_DIR` is set so `w3`/willbe writes build artifacts to the plugin volume instead of `/workspace/` — critical when `/workspace/` is bind-mounted `:ro` (Fix(BUG-001)). `cmd_list()` also sets `CARGO_TARGET_DIR=/tmp/will_test_targets` so `.list` benefits from the same cached artifacts. A second binary plugin (e.g., `gh`) requires adding a parallel `bin_plugin_2` field to `runbox.yml` and handling it in `plugins.sh` — no changes to `runbox-run`.
