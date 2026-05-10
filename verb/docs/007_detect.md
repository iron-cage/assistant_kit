# Verb: `detect`

- **Kind:** meta
- **Availability:** universal
- **`--dry-run`:** not supported (output is the result)

### Command

```bash
printf 'ecosystem:  cargo\n'
printf 'signal:     Cargo.toml\n'
printf 'confidence: inferred\n'
```

Fixed output — identical across all workspace modules.

### Notes

Protocol discovery verb — reports the project ecosystem for tooling integration. Output is machine-readable: key-value lines separated by `:`, newline-terminated.

Fields:
- `ecosystem` — the build toolchain/package manager (`cargo` for all workspace modules)
- `signal` — the file whose presence indicates this ecosystem (`Cargo.toml`)
- `confidence` — how the detection was determined: `inferred` means from file presence, not an explicit declaration

`confidence: inferred` distinguishes this from a future `declared` confidence level where the ecosystem is stated in a config file. Currently all modules are `inferred`.

No `--dry-run` flag: no side-effectful operation to preview — the `printf` output IS the detection result.

`detect` is **identical across all cargo modules** — the output never varies by module name because it describes the ecosystem, not the module.

### Example

```bash
# Any cargo module (output is always identical)
./verb/detect
```
Output:
```
ecosystem:  cargo
signal:     Cargo.toml
confidence: inferred
```

Script body:
```bash
#!/usr/bin/env bash
# do detect — identify the project ecosystem (protocol meta-verb)
set -euo pipefail
printf 'ecosystem:  cargo\n'
printf 'signal:     Cargo.toml\n'
printf 'confidence: inferred\n'
```
