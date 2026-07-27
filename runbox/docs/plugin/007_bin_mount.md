# Plugin: `bin_mount`

- **Status:** ✅ Configured — defined in `runbox/plugins.sh`; configured via `runbox.yml`
- **Controls:** Host binary injected into container, read-only, no working volume
- **Mechanism:** `which name` on host → bind-mount binary `:ro` into container; implemented in `_plugin_test_args` in `runbox/plugins.sh`

### Notes

Configured via `runbox.yml` key `bin_mount: name:/container/path`. Current use: `claude:/usr/local/bin/claude` — lets `.test` spawn the real `claude` CLI already installed and authenticated on the host, instead of installing and authenticating a separate copy inside the image. Plugin logic lives entirely in `runbox/plugins.sh` — core `runbox-run` has no plugin knowledge.

Distinct from `bin_plugin` (→ `001_bin_plugin.md`): `bin_plugin` also provisions a named working volume and sets `CARGO_TARGET_DIR` for binaries that compile their own output (e.g. `w3`). `bin_mount` skips both — appropriate for a binary invoked read-only that produces no build artifacts of its own. A crate using both `bin_plugin` and `bin_mount` at once is safe: each contributes independent `-v` mounts, and only `bin_plugin` touches `CARGO_TARGET_DIR`.

Resolution happens at `.test` invocation time via `which`, not at image build time — the container always sees whatever `claude` binary/version is currently on the host's `PATH` (including self-updates), and the image itself never embeds a `claude` binary or its credentials. Credentials/config are supplied separately via the existing `plugin_mount: ~/.claude:/workspace/.claude:directory` (→ `002_plugin_mount.md`), so `claude` inside the container authenticates exactly as it does on the host.

Required, not optional: if the named binary is absent from the host `PATH`, `.test` fails fast with `error: bin_mount '<name>' not found on PATH` rather than letting every dependent test fail individually downstream with a less specific error.

### Example

```yaml
bin_mount: claude:/usr/local/bin/claude
```
`cmd_test()` resolves `which claude` on the host, then adds:
```bash
-v /home/user/.local/bin/claude:/usr/local/bin/claude:ro
```
Inside the container, `claude` is callable at `/usr/local/bin/claude` — already on `PATH` in the base image (same as the `bin_plugin`-mounted `w3`). The host path is typically a symlink to a versioned, self-updating binary (e.g. `~/.local/share/claude/versions/X.Y.Z`); the bind-mount's source-path resolution follows the symlink to the underlying regular file, so the mount always tracks whatever version is currently active on the host — no image rebuild required when `claude` self-updates.
