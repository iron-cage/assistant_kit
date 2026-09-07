# Tool: Artifact

Publish an HTML or Markdown file as a shareable artifact on claude.ai.

### Category

Publishing

### Permission Required

Yes

### Description

Publishes an HTML or Markdown file as an artifact — a private, interactive page
on claude.ai that can be shared within your organization. The published artifact
gets a unique URL.

### Availability

Requires Team or Enterprise plan and `/login` authentication. Not available on
Bedrock, Vertex AI, or Foundry.

### Parameters

❌ **`filePath` refuted** — the doc previously named the file parameter `filePath` (camelCase). Confirmed wrong: v2.1.220's Artifact tool schema names it `file_path` (snake_case; "Path to an .html or .md file to render" — 1 exact hit). The full real parameter set is substantially richer than a single required file path:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `file_path` | string | to publish | Path to an `.html` or `.md` file to render. Required unless `action: "list"`. Wrapped in an HTML skeleton at publish time for `.html` files. |
| `action` | enum | ❌ | `"publish"` (default, omit to publish `file_path`) or `"list"` (enumerate the user's published artifacts — the only other parameters valid alongside it are `limit`/`scope`). |
| `title` | string | ❌ | Title shown in the browser tab and gallery. A `<title>` tag in the HTML itself takes precedence; this is a fallback only. Markdown pages keep their filename identity instead. |
| `description` | string | ❌ | One-sentence subtitle shown on the gallery card (max 1000 chars). |
| `favicon` | string | required to publish | Browser-tab icon: one or two emoji (e.g. `"📊"`), no markup (max 32 chars). Keep stable across redeploys. |
| `url` | string | ❌ | Existing artifact URL to update in place, so an update targets the same published page instead of minting a new one. Must be an artifact the user owns. |
| `capabilities` | object | ❌ | Runtime capabilities the page declares, as `{name: config}`. Omitting the field on a redeploy keeps the stored declaration; `{}` clears it. |
| `contract` | string | ❌ | The artifact's runtime version — `"latest"` or a specific `major.minor.patch`. Omit to keep the current version. |
| `force` | boolean | ❌ | Last-resort overwrite that discards another session's published version on a version conflict. Defaults to `false`. |
| `label` | string | ❌ | Short human-readable name for this version (max 60 chars), shown in the version picker. |
| `scope` | enum | ❌ | `action: "list"` only — `"mine"` (default), `"shared"`, or `"all"`. |
| `limit` | integer | ❌ | `action: "list"` only — maximum artifacts to return (1-50, default 25). |

**Verify:** these 12 field names and their exact description text are drawn from the Artifact tool's own live schema and cross-checked as literal substrings against the pinned v2.1.220 binary (e.g. `grep -c -F "Path to an .html or .md file to render" ~/.local/share/claude/versions/2.1.220` → 1; `grep -c -F "filePath" ...` → present only as generic unrelated identifier noise elsewhere in the bundle, never paired with this tool's own description strings).

### Since

v2.0+ (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master tool table |
| doc | [037_share_onboarding_guide.md](037_share_onboarding_guide.md) | ShareOnboardingGuide — share guide files externally (related publishing tool) |
