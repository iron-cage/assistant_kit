# Subcommand: gateway

Run the enterprise auth/telemetry gateway.

### Usage

```
claude gateway [options]
```

### Options

| Flag | Description |
|------|-------------|
| `--config <path>` | Path to gateway YAML config |
| `-h`, `--help` | Display help for command |

### Sub-subcommands

None.

### Description

Runs the enterprise authentication and telemetry gateway as a foreground
process. This is a deployment-side command rather than a developer-session
command: it stands up the service that enterprise-managed Claude Code clients
authenticate through, rather than doing anything to the local session.

Configuration is supplied as a YAML file via `--config`. The config schema is
not documented in `claude gateway --help` and has not been captured here — see
the Verification note below before relying on any assumed shape.

Distinct from the unrelated appearances of the word "gateway" elsewhere in the
changelog, which refer to *third-party Anthropic-compatible proxy gateways*
addressed by `ANTHROPIC_BASE_URL` (for example the `/model` picker listing
models from a gateway's `/v1/models` endpoint). Those are a client-side
concern; this subcommand runs a server.

### Since

Unverified. No changelog entry in the `version/` collection records the
introduction of this subcommand; the word "gateway" appears in earlier releases
only in the unrelated proxy-gateway sense described above. Present in v2.1.220.

### Verification

```bash
claude gateway --help          # → Usage: claude gateway [options]
claude --help | grep gateway   # → listed under Commands:
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master subcommand table |
| doc | [002_auth.md](002_auth.md) | Client-side authentication management |
