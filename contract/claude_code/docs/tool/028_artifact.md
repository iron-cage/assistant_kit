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

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `filePath` | string | yes | Path to the HTML or Markdown file to publish |

### Since

v2.0+ (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master tool table |
| doc | [../params/readme.md](../params/readme.md) | `CLAUDE_CODE_DISABLE_ARTIFACT` env var |
