# Workflow Scenario :: 3. Scripted Health Check

Use JSON output for pipeline integration in monitoring scripts.

```bash
#!/bin/bash
# health_check.sh — exit non-zero if token is not valid

status=$( clp .token.status format::json )
state=$( echo "$status" | jq -r '.status' )

case "$state" in
  valid)
    echo "ok"
    exit 0
    ;;
  expiring_soon)
    echo "warning: token expiring soon"
    exit 0
    ;;
  expired)
    echo "error: token expired"
    exit 1
    ;;
esac
```

**When to use:** CI/CD pipelines, cron monitoring, pre-flight checks before batch operations.
