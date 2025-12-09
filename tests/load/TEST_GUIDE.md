# Load Testing Guide - Quick Reference

This is a quick reference guide for running k6 load tests on the lighter-auth service.

## Quick Start

```bash
# 1. Start the service
cargo run --features postgres

# 2. Run a quick smoke test (2 minutes)
k6 run tests/load/quick-test.js

# 3. Run the standard load test (9 minutes)
k6 run tests/load/k6_script.js
```

## Test Files Overview

| File | Duration | Max VUs | Purpose | When to Use |
|------|----------|---------|---------|-------------|
| `quick-test.js` | 2 min | 20 | Quick smoke test | Development, CI/CD |
| `k6_script.js` | 9 min | 100 | Standard load test | Regular testing |
| `spike-test.js` | 10 min | 200 | Traffic spike simulation | Auto-scaling validation |
| `stress-test.js` | 15 min | 250 | Breaking point identification | Capacity planning |
| `soak-test.js` | 2+ hours | 50 | Long-term stability | Memory leak detection |

## Test Selection Guide

### Development Workflow
```bash
# During development - fast feedback
k6 run tests/load/quick-test.js
```

### Before Merging PR
```bash
# Standard load test
k6 run tests/load/k6_script.js
```

### Before Production Deploy
```bash
# Run all critical tests
./tests/load/run-all-tests.sh --quick
```

### Performance Regression Testing
```bash
# Standard load + spike test
k6 run tests/load/k6_script.js
k6 run tests/load/spike-test.js
```

### Capacity Planning
```bash
# Stress test to find limits
k6 run tests/load/stress-test.js
```

### Stability Validation
```bash
# Soak test (overnight/weekend)
k6 run tests/load/soak-test.js
# Or 30-minute version
k6 run tests/load/soak-test.js -e DURATION=30m
```

## Test Results Interpretation

### ✅ Good Results
- All thresholds passing (green checkmarks)
- Success rates > 98%
- p(95) response times < 500ms
- Stable throughout test duration
- Low failed operations count

### ⚠️ Warning Signs
- Success rates 90-95%
- p(95) response times 500-800ms
- Gradual performance degradation
- Error rate increasing over time

### ❌ Critical Issues
- Success rates < 90%
- p(95) response times > 1000ms
- Thresholds failing
- High error rates
- System not recovering after load

## Common Commands

```bash
# Run with custom base URL
k6 run -e BASE_URL=https://staging.example.com tests/load/k6_script.js

# Save results to JSON
k6 run --out json=results.json tests/load/k6_script.js

# Run with custom VU count (override stages)
k6 run --vus 50 --duration 5m tests/load/k6_script.js

# Verbose output for debugging
k6 run --verbose tests/load/k6_script.js

# HTTP debug logging
k6 run --http-debug tests/load/k6_script.js
```

## Monitoring During Tests

### System Resources
```bash
# Monitor CPU and memory
watch -n 1 'ps aux | grep lighter-auth'

# Monitor network connections
watch -n 1 'netstat -an | grep :8080 | wc -l'

# Monitor disk I/O
iostat -x 1
```

### Database
```bash
# PostgreSQL connections
watch -n 1 'psql -U lighter -d lighter_auth -c "SELECT count(*) FROM pg_stat_activity;"'

# Check for locks
psql -U lighter -d lighter_auth -c "SELECT * FROM pg_locks WHERE NOT granted;"
```

### Application Logs
```bash
# Follow logs (if using file logging)
tail -f /var/log/lighter-auth.log

# Or journalctl
journalctl -u lighter-auth -f
```

## Performance Baselines

Expected performance on modern hardware (i7 8-core, 16GB RAM):

| Metric | Expected Value |
|--------|----------------|
| Throughput | 35-40 req/s |
| Success Rate | > 98% |
| Login p(95) | 150-250ms |
| Auth Check p(95) | 50-100ms (cached) |
| User Creation p(95) | 250-350ms |
| Overall p(95) | 250-350ms |
| Overall p(99) | 500-800ms |

## Troubleshooting Quick Reference

| Issue | Likely Cause | Solution |
|-------|--------------|----------|
| Connection refused | Service not running | `cargo run --features postgres` |
| High error rate | Database pool exhausted | Increase pool size, check queries |
| Slow response times | CPU/memory constrained | Add resources, optimize code |
| Cache misses | Cache TTL too short | Increase TTL or use Redis |
| Database locks | SQLite (single writer) | Use PostgreSQL |
| Memory growth | Memory leak | Profile with valgrind, check connections |

## CI/CD Integration

### GitHub Actions Example
```yaml
- name: Load Test
  run: k6 run tests/load/quick-test.js
```

### GitLab CI Example
```yaml
load_test:
  script:
    - k6 run tests/load/quick-test.js
```

## Next Steps

1. **First Time**: Start with `quick-test.js` to validate setup
2. **Regular Testing**: Use `k6_script.js` as standard test
3. **Before Deploy**: Run `run-all-tests.sh --quick`
4. **Performance Issues**: Use `stress-test.js` to find limits
5. **Stability Concerns**: Run `soak-test.js` overnight

## Additional Resources

- Full documentation: `README.md`
- Main test script: `k6_script.js`
- Test runner: `run-all-tests.sh`
- k6 docs: https://k6.io/docs/

---

**Quick Tips:**

1. Always run tests in a test environment first
2. Monitor system resources during tests
3. Compare results across test runs
4. Use quick-test for rapid feedback
5. Run soak test periodically to catch leaks
6. Save results for historical comparison
7. Adjust thresholds based on your SLAs

**Remember:** These are load tests, not production traffic. Always test in isolated environments!
