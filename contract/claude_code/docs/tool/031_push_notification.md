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
| `message` | string | yes | Notification body text (keep under 200 characters; mobile OSes truncate) |
| `status` | string (const `"proactive"`) | yes | Fixed marker value identifying the notification as agent-initiated |

### Since

v2.1.110

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master tool table |
