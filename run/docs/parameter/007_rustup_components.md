# Parameter: `rustup_components`

- **Status:** ✅ Configured — via `runbox.yml`; default: `clippy`
- **Current State:** `clippy`
- **Where It Flows:** `runbox.yml rustup_components:` → `--build-arg RUSTUP_COMPONENTS` → `RUN rustup component add $RUSTUP_COMPONENTS` in test stage

### Notes

Space-separated — `rustup component add` accepts multiple components in one invocation. `clippy` is required by `w3 .test level::3` (`cargo clippy -D warnings`). Adding `rustfmt` or `llvm-tools-preview` requires only a `runbox.yml` edit and `.build`.

### Example

Adding `rustfmt` for format-check tests alongside the required `clippy`:
```yaml
rustup_components: clippy rustfmt
```
`docker-run` passes `--build-arg RUSTUP_COMPONENTS=clippy rustfmt` → dockerfile runs `rustup component add clippy rustfmt` in a single invocation. Both components are available in the test stage after rebuild.
