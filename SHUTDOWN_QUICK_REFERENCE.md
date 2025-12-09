# Graceful Shutdown - Quick Reference

## Configuration

```bash
# Via environment variable
export LIGHTER_AUTH__APP__SHUTDOWN_TIMEOUT=30

# Via config file (config/default.toml)
[app]
shutdown_timeout = 30
```

## Testing

```bash
# Automated test
./test_shutdown.sh

# Manual test - Ctrl+C
cargo run
# Press Ctrl+C

# Manual test - SIGTERM
cargo run &
kill -TERM $!
```

## Expected Logs

```
INFO lighter_auth: Received shutdown signal, initiating graceful shutdown and draining in-flight requests shutdown_timeout_seconds=30
INFO lighter_auth: Graceful shutdown completed successfully
```

## Docker

```dockerfile
# Correct: Exec form
CMD ["lighter-auth"]

# Wrong: Shell form (doesn't forward signals)
CMD lighter-auth
```

```yaml
# docker-compose.yml
services:
  auth:
    stop_grace_period: 35s  # > shutdown_timeout
```

## Kubernetes

```yaml
spec:
  terminationGracePeriodSeconds: 35  # > shutdown_timeout
  containers:
  - name: auth
    env:
    - name: LIGHTER_AUTH__APP__SHUTDOWN_TIMEOUT
      value: "30"
```

## Troubleshooting

| Problem | Solution |
|---------|----------|
| App doesn't shutdown | Check CMD uses exec form `["app"]` |
| Requests fail during deploy | Increase shutdown_timeout |
| Timeout warnings | Check for slow requests |
| Database errors | Ensure queries timeout properly |

## Best Practices

- Set `shutdown_timeout` to 10-60 seconds
- Set container grace period to `shutdown_timeout + 5s`
- Monitor shutdown logs for issues
- Test with both SIGTERM and SIGINT
- Use rolling deployments in production

## Key Signals

- **SIGTERM** - Graceful shutdown (production)
- **SIGINT** - Graceful shutdown (development/Ctrl+C)
- **SIGKILL** - Force kill (avoid, no cleanup)

## Files

- `src/main.rs` - Implementation
- `CLAUDE.md` - Full documentation (Section 10)
- `test_shutdown.sh` - Automated testing
- `config/default.toml` - Configuration

## Quick Health Check

```bash
# Start server
cargo run &
PID=$!

# Send SIGTERM
sleep 3
kill -TERM $PID

# Check logs for graceful shutdown
wait $PID
echo $?  # Should be 0
```
