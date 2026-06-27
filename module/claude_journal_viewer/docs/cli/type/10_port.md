# CLI Type: Port

TCP port number for the embedded HTTP server.

- **Kind:** Semantic
- **Fundamental:** Integer
- **Key Constraint:** 0-65535

### Values

| Range | Behavior |
|-------|----------|
| `0` | OS assigns an available ephemeral port |
| `1-1023` | Privileged ports (requires root on Linux) |
| `1024-65535` | Unprivileged ports (recommended) |
| `8411` | Default `clj` viewer port |

### Validation

- Parsed as `u16` (unsigned 16-bit integer)
- Values > 65535 cause exit 1
- Bind failure (port in use) causes exit 1

### Referenced Parameters

| # | Parameter |
|---|-----------|
| 15 | [`port`](../param/15_port.md) |
