# Verb: `verbs`

- **Kind:** meta
- **Availability:** universal
- **`--dry-run`:** not supported (output is the result)

### Command

```bash
printf '%-8s %-11s %s\n' <verb> <status> <command>
```

One `printf` line per verb. Columns: verb name (8 chars), status (11 chars), command string. Status values: `available`, `unavailable`, `built-in`.

### Notes

Protocol introspection verb — reports what actions the module supports. Output is machine-readable: fixed-width columns, newline-separated, no header. Tooling can parse the status column to decide whether to invoke a verb.

`verbs` and `detect` always show `built-in` status — they are defined by the protocol itself and always present, so they report no specific command.

`run` shows `unavailable` for library modules and `available` with the binary invocation for binary modules. All other canonical verbs are always `available`.

No `--dry-run` flag: there is no side-effectful operation to preview — the `printf` output IS the introspection result.

### Example

Binary module (`claude_profile`):
```
build    available   cargo build -p claude_profile
test     available   w3 .test level::3
clean    available   cargo clean -p claude_profile
run      available   cargo run -p claude_profile --bin clp
lint     available   cargo clippy -p claude_profile --all-features -- -D warnings
verify   available   w3 .test level::4
verbs    built-in    -
detect   built-in    -
```

Library module (`claude_storage_core`):
```
build    available    cargo build -p claude_storage_core
test     available    w3 .test level::3
clean    available    cargo clean -p claude_storage_core
run      unavailable  -
lint     available    cargo clippy -p claude_storage_core --all-features -- -D warnings
verify   available    w3 .test level::4
verbs    built-in     -
detect   built-in     -
```

Script body (binary module):
```bash
#!/usr/bin/env bash
# do verbs — list available verbs for this project (protocol meta-verb)
set -euo pipefail
printf '%-8s %-10s %s\n' build   available "cargo build -p claude_profile"
printf '%-8s %-10s %s\n' test    available "w3 .test level::3"
printf '%-8s %-10s %s\n' clean   available "cargo clean -p claude_profile"
printf '%-8s %-10s %s\n' run     available "cargo run -p claude_profile --bin clp"
printf '%-8s %-10s %s\n' lint    available "cargo clippy -p claude_profile --all-features -- -D warnings"
printf '%-8s %-10s %s\n' verify  available "w3 .test level::4"
printf '%-8s %-10s %s\n' verbs   built-in  "-"
printf '%-8s %-10s %s\n' detect  built-in  "-"
```
