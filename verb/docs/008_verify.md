# Verb: `verify`

- **Kind:** canonical
- **Availability:** universal
- **`--dry-run`:** `w3 .test level::4` (most modules) / `./verb/test && cargo +nightly udeps … && cargo +nightly audit` (workspace root, `claude_runner`, `claude_version`)

### Command

Most modules:

```bash
w3 .test level::4
```

Level 4 runs: nextest (all features, warnings-as-errors) + doc tests + clippy (-D warnings) + `cargo +nightly udeps` (unused dependency detection) + `cargo +nightly audit` (security vulnerability scan of `Cargo.lock`). It executes host-side — the container-only test invariant blocks its nextest stage on a bare host, so this form only completes where host execution is authorized (`VERB_LAYER=l0` path).

Workspace root, `claude_runner`, `claude_version` — container-chaining variant:

```bash
./verb/test                                       # container suite via runbox .live
cargo +nightly udeps --all-targets --all-features # host-side, no test execution
cargo +nightly audit                              # host-side; skipped when no Cargo.lock
```

The test stage runs inside the container; udeps and audit are host-side dependency hygiene and never execute tests.

### Notes

`verify` is a superset of `test`. Where `test` gives fast per-PR feedback, `verify` is the full pre-push gate — everything `test` does plus dependency hygiene and security analysis.

`udeps` catches dependencies declared in `Cargo.toml` that are never actually used. `audit` cross-references `Cargo.lock` against the RustSec advisory database. Both require nightly.

Library crates skip the audit step automatically when no `Cargo.lock` is present.

`--dry-run` prints the delegated command chain and exits 0 — no checks run.

### Example

```bash
./verb/verify              # root: container test + udeps + audit; module: w3 .test level::4
./verb/verify --dry-run    # prints the command chain
```

Relation to `test`:
```
test   → runbox .live → test.d/l1   (nextest + doc tests + clippy, in container)
verify → test + udeps + audit       (dependency hygiene added, host-side)
```
