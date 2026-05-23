# Format :: 2. json

- **ID:** F02
- **Trigger:** `format::json`
- **Scope:** All format-capable commands: `.accounts`, `.token.status`, `.paths`, `.usage`, `.credentials.status`, `.account.limits`

### Structure

Single-line JSON. All fields included regardless of field-presence parameters (`sub::`, `tier::`, etc.).

- **Single-record commands** (`.credentials.status`, `.token.status`, `.paths`, `.account.limits`): JSON object `{...}`
- **Multi-record commands** (`.accounts`, `.usage`): JSON array `[{...}, {...}]`

Error rows in `.usage` appear as `{"account":"...","error":"..."}` objects in the array.

### Rendering Mechanism

`serde_json` serialization via `data_fmt` JSON renderer — no pretty-printing; all fields serialized regardless of field-presence toggles.

### Example

```bash
clp .credentials.status format::json
# {"subscription":"max","tier":"default_claude_max_20x","token":"valid","expires_in_secs":26640,"email":"alice@acme.com","account":"alice@acme.com","file":"/home/user/.claude/.credentials.json","saved":2}

clp .account.limits format::json
# {"session_pct":62,"session_reset_secs":6480,"weekly_all_pct":41,"weekly_all_reset_secs":302400,"weekly_sonnet_pct":38,"weekly_sonnet_reset_secs":302400}

clp .usage format::json
# [
#   {"account":"i12@wbox.pro","expires_at_ms":1748033040000,"session_5h_left_pct":86,...},
#   {"account":"i6@wbox.pro","expires_at_ms":1748028720000,"session_5h_left_pct":100,...}
# ]
```

**Notes:**
- `format::json` combined with `live::1` exits 1 before any fetch (incompatible combination).
- Field-presence params (`sub::0`, `tier::0`, etc.) are ignored in JSON mode — all fields always appear.
