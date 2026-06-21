# Tool: PushNotification

Send a desktop or phone push notification.

### Category

Notification

### Permission Required

No

### Description

Sends a desktop notification, and a phone push notification when Remote Control
is connected. Useful for long-running or scheduled tasks to reach users who have
stepped away from the terminal.

### Availability

Not available on Bedrock, Vertex AI, or Foundry.

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `title` | string | yes | Notification title |
| `body` | string | yes | Notification body text |

### Since

v2.0+ (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master tool table |
