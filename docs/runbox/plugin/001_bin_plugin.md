# Plugin: `bin_plugin`

- **Status:** ⚠️ Partial — configurable in `runbox.yml`; capacity hardcoded at one instance
- **Controls:** Host binary injected into container with a working volume
- **Mechanism:** `which name` on host → bind-mount binary `:ro` into container; named Docker volume for working directory

### Notes

Configured via `runbox.yml` key `bin_plugin: name:/container/path`. Current use: `w3:/usr/local/bin/w3`. A second binary plugin slot requires code changes to `docker-run`.

### Example

```yaml
bin_plugin: w3:/usr/local/bin/w3
bin_plugin_volume: /tmp/will_test_targets
```
`cmd_test()` resolves `which w3` on the host, then builds:
```bash
-v workspace_test_plugin_targets:/tmp/will_test_targets
-v /usr/local/bin/w3:/usr/local/bin/w3:ro
```
Inside the container `w3` is callable at `/usr/local/bin/w3` exactly as on the host. The `workspace_test_plugin_targets` volume persists `w3`'s compilation artifacts across `.test` invocations — repeated calls reuse prior builds instead of recompiling. In `cmd_list()`, `CARGO_TARGET_DIR=/tmp/will_test_targets` redirects nextest's artifact path into the same volume so `.list` also benefits. A second binary plugin (e.g., `gh`) requires adding a parallel `bin_plugin_2` field and corresponding code in `docker-run`.
