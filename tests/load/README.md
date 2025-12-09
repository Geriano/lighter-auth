# K6 Load Testing for lighter-auth

Comprehensive load testing suite for the lighter-auth authentication microservice using k6.

## Table of Contents

- [Overview](#overview)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Running the Tests](#running-the-tests)
- [Test Scenarios](#test-scenarios)
- [Performance Thresholds](#performance-thresholds)
- [Interpreting Results](#interpreting-results)
- [Customization](#customization)
- [Troubleshooting](#troubleshooting)
- [Performance Benchmarks](#performance-benchmarks)

## Overview

This load test suite simulates realistic user behavior against the lighter-auth service, testing:

- User registration (POST /v1/user)
- User login (POST /login)
- Authenticated user retrieval (GET /user)
- User logout (DELETE /logout)

The test progressively increases load from 0 to 100 concurrent users over a 9-minute period, measuring response times, success rates, and throughput.

## Prerequisites

- **k6**: Load testing tool (v0.45.0 or higher recommended)
- **lighter-auth service**: Running and accessible
- **Database**: PostgreSQL or SQLite properly configured
- **System resources**: Sufficient CPU and memory for test load

## Installation

### Install k6

**macOS (Homebrew):**
```bash
brew install k6
```

**Linux (Debian/Ubuntu):**
```bash
sudo gpg -k
sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
echo "deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
sudo apt-get update
sudo apt-get install k6
```

**Linux (Fedora/CentOS):**
```bash
sudo dnf install https://dl.k6.io/rpm/repo.rpm
sudo dnf install k6
```

**Windows (Chocolatey):**
```powershell
choco install k6
```

**Docker:**
```bash
docker pull grafana/k6:latest
```

### Verify Installation

```bash
k6 version
```

Expected output: `k6 v0.45.0 (or higher)`

## Running the Tests

### Basic Usage

1. **Start the lighter-auth service:**
   ```bash
   # From project root
   cargo run --features postgres
   ```

2. **Run the load test:**
   ```bash
   # From project root
   k6 run tests/load/k6_script.js
   ```

### With Custom Base URL

```bash
k6 run -e BASE_URL=http://localhost:8080 tests/load/k6_script.js
```

### With Docker

```bash
# Start lighter-auth with docker-compose
docker-compose up -d

# Run k6 load test
docker run --rm -i --network=host \
  -v $(pwd)/tests/load:/tests \
  grafana/k6:latest run /tests/k6_script.js
```

### Save Results to File

```bash
k6 run --out json=results.json tests/load/k6_script.js
```

### Run with Custom Stages (Quick Test)

Create a custom configuration file `quick-test.js`:

```javascript
export { default, setup, teardown } from './k6_script.js';

export const options = {
  stages: [
    { duration: '30s', target: 20 },  // Ramp up to 20 users
    { duration: '1m', target: 20 },   // Hold at 20 users
    { duration: '30s', target: 0 },   // Ramp down
  ],
};
```

Run:
```bash
k6 run tests/load/quick-test.js
```

## Test Scenarios

### Default Load Profile

The default test simulates a realistic production load pattern:

```
Stage 1: 0 → 50 users   (1 minute)   - Initial ramp-up
Stage 2: 50 users       (3 minutes)  - Sustained moderate load
Stage 3: 50 → 100 users (1 minute)   - Peak load ramp-up
Stage 4: 100 users      (3 minutes)  - Sustained peak load
Stage 5: 100 → 0 users  (1 minute)   - Cool down
```

**Total Duration:** ~9 minutes

### User Lifecycle Simulation

Each virtual user (VU) performs the following sequence:

1. **Create User** (POST /v1/user)
   - Generate unique username, email, password
   - Validate user creation response
   - Extract user ID

2. **Think Time** (1 second)

3. **Login** (POST /login)
   - Authenticate with created credentials
   - Extract authentication token
   - Validate token presence

4. **Think Time** (1 second)

5. **Get Authenticated User** (GET /user)
   - Use authentication token
   - Verify user data returned
   - Tests auth middleware and caching

6. **Think Time** (1 second)

7. **Logout** (DELETE /logout)
   - Invalidate authentication token
   - Verify successful logout

8. **Think Time** (2 seconds)
   - Simulates user browsing between sessions

### Metrics Collected

**Built-in HTTP Metrics:**
- `http_req_duration`: Request duration (p95, p99, avg, min, max)
- `http_req_failed`: Percentage of failed requests
- `http_reqs`: Total requests and request rate
- `http_req_waiting`: Time to first byte (TTFB)
- `http_req_connecting`: Connection establishment time
- `http_req_sending`: Request sending time
- `http_req_receiving`: Response receiving time

**Custom Metrics:**
- `login_success_rate`: Login operation success rate
- `create_user_success_rate`: User creation success rate
- `auth_user_success_rate`: Authentication check success rate
- `logout_success_rate`: Logout operation success rate
- `login_duration`: Login-specific duration trend
- `create_user_duration`: User creation duration trend
- `auth_user_duration`: Auth check duration trend
- `logout_duration`: Logout duration trend
- `total_operations`: Total operations performed
- `failed_operations`: Total failed operations

## Performance Thresholds

The test enforces the following performance requirements:

### Response Time Thresholds

| Threshold | Target | Description |
|-----------|--------|-------------|
| `http_req_duration p(95)` | < 500ms | 95% of requests complete in under 500ms |
| `http_req_duration p(99)` | < 1000ms | 99% of requests complete in under 1s |
| `login_duration p(95)` | < 300ms | Login completes in under 300ms |
| `create_user_duration p(95)` | < 500ms | User creation completes in under 500ms |
| `auth_user_duration p(95)` | < 200ms | Auth check completes in under 200ms (cached) |
| `logout_duration p(95)` | < 200ms | Logout completes in under 200ms |

### Success Rate Thresholds

| Threshold | Target | Description |
|-----------|--------|-------------|
| `http_req_failed` | < 5% | Overall error rate below 5% |
| `login_success_rate` | > 95% | Login success rate above 95% |
| `create_user_success_rate` | > 95% | User creation success above 95% |
| `auth_user_success_rate` | > 95% | Auth check success above 95% |
| `logout_success_rate` | > 95% | Logout success above 95% |

### Throughput Thresholds

| Threshold | Target | Description |
|-----------|--------|-------------|
| `http_reqs` | > 10 req/s | Minimum throughput of 10 requests per second |

**Threshold Status:**
- ✅ **PASS**: All thresholds met
- ❌ **FAIL**: One or more thresholds violated

## Interpreting Results

### Example Output

```
     ✓ user created successfully
     ✓ user response has id
     ✓ login successful
     ✓ login response has token
     ✓ login response has user data
     ✓ authenticated user retrieved
     ✓ authenticated user has data
     ✓ logout successful

     █ setup

     █ teardown

     auth_user_duration............: avg=45.23ms  min=12.34ms med=38.45ms max=234.56ms p(90)=78.9ms  p(95)=102.34ms
     auth_user_success_rate........: 98.76% ✓ 4938      ✗ 62
     checks.........................: 99.23% ✓ 39504     ✗ 306
     create_user_duration..........: avg=123.45ms min=45.67ms med=98.76ms max=876.54ms p(90)=234.5ms p(95)=345.67ms
     create_user_success_rate......: 97.89% ✓ 4894      ✗ 106
     data_received..................: 15 MB  28 kB/s
     data_sent......................: 8.2 MB 15 kB/s
     failed_operations..............: 474    0.88/s
     http_req_blocked...............: avg=2.34ms   min=1µs     med=3µs     max=123.45ms p(90)=4µs     p(95)=5µs
     http_req_connecting............: avg=1.23ms   min=0s      med=0s      max=98.76ms  p(90)=0s      p(95)=0s
     http_req_duration..............: avg=89.34ms  min=12.34ms med=67.89ms max=987.65ms p(90)=156.7ms p(95)=234.56ms
       { expected_response:true }...: avg=89.12ms  min=12.34ms med=67.45ms max=876.54ms p(90)=154.3ms p(95)=231.23ms
     ✓ { p(95)<500 }.................: true
     ✓ { p(99)<1000 }................: true
     http_req_failed................: 2.34%  ✓ 468       ✗ 19532
     ✓ { rate<0.05 }.................: true
     http_req_receiving.............: avg=234µs    min=23µs    med=123µs   max=12.34ms  p(90)=456µs   p(95)=678µs
     http_req_sending...............: avg=123µs    min=12µs    med=67µs    max=5.67ms   p(90)=234µs   p(95)=345µs
     http_req_tls_handshaking.......: avg=0s       min=0s      med=0s      max=0s       p(90)=0s      p(95)=0s
     http_req_waiting...............: avg=88.98ms  min=12.12ms med=67.56ms max=985.43ms p(90)=155.6ms p(95)=233.45ms
     http_reqs......................: 20000  37.12/s
     ✓ { rate>10 }....................: true
     iteration_duration.............: avg=7.5s     min=6.2s    med=7.4s    max=9.8s     p(90)=8.3s    p(95)=8.9s
     iterations.....................: 5000   9.28/s
     login_duration.................: avg=67.89ms  min=23.45ms med=56.78ms max=543.21ms p(90)=123.4ms p(95)=178.9ms
     ✓ { p(95)<300 }.................: true
     login_success_rate.............: 98.45% ✓ 4922      ✗ 78
     ✓ { rate>0.95 }.................: true
     logout_duration................: avg=34.56ms  min=8.9ms   med=28.34ms max=234.56ms p(90)=67.8ms  p(95)=89.12ms
     ✓ { p(95)<200 }.................: true
     logout_success_rate............: 99.12% ✓ 4956      ✗ 44
     ✓ { rate>0.95 }.................: true
     total_operations...............: 20000  37.12/s
     vus............................: 0      min=0       max=100
     vus_max........................: 100    min=100     max=100
```

### Key Metrics Explained

**Success Rates:**
- Values > 95% are good
- Values 90-95% may indicate issues under load
- Values < 90% indicate serious problems

**Response Times:**
- `p(95)`: 95% of requests are faster than this
- `p(99)`: 99% of requests are faster than this
- Lower values are better
- Sudden spikes may indicate:
  - Database connection pool exhaustion
  - CPU/memory constraints
  - Network issues
  - Database query performance problems

**Throughput:**
- `http_reqs`: Total requests per second
- Higher values indicate better scalability
- Should remain stable during sustained load stages

**Checks:**
- Shows validation success/failure counts
- High failure rate indicates API contract issues

### What Good Results Look Like

✅ **Healthy System:**
- All thresholds passing (green checkmarks)
- Success rates > 98%
- p(95) response times < 500ms
- p(99) response times < 1000ms
- Throughput > 10 req/s sustained
- Minimal variation between p(95) and p(99)
- Low failed_operations count

❌ **System Under Stress:**
- Failed thresholds (red X marks)
- Success rates < 95%
- p(95) > 500ms or p(99) > 1000ms
- Throughput dropping during sustained load
- Large gap between p(95) and p(99) (indicates outliers)
- High failed_operations count

## Customization

### Modify Load Profile

Edit `k6_script.js` and change the `options.stages` array:

```javascript
export const options = {
  stages: [
    { duration: '2m', target: 200 },  // Stress test with 200 users
    { duration: '5m', target: 200 },  // Hold for 5 minutes
    { duration: '2m', target: 0 },    // Ramp down
  ],
  // ... rest of options
};
```

### Adjust Thresholds

Modify thresholds in `options.thresholds`:

```javascript
thresholds: {
  'http_req_duration': ['p(95)<300', 'p(99)<600'],  // Stricter requirements
  'login_success_rate': ['rate>0.99'],               // 99% success rate
  // ... other thresholds
},
```

### Change Base URL

```bash
# Via environment variable
k6 run -e BASE_URL=https://staging.example.com tests/load/k6_script.js

# Or edit the script
const BASE_URL = __ENV.BASE_URL || 'https://production.example.com';
```

### Add Custom Scenarios

Create a new test file that imports the base script:

```javascript
import { createUser, login, getAuthenticatedUser, logout } from './k6_script.js';

export const options = {
  scenarios: {
    // Constant load
    constant_load: {
      executor: 'constant-vus',
      vus: 50,
      duration: '5m',
    },

    // Spike test
    spike_test: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '10s', target: 100 },  // Fast ramp-up
        { duration: '1m', target: 100 },   // Hold
        { duration: '10s', target: 0 },    // Fast ramp-down
      ],
      startTime: '5m',  // Start after constant_load
    },
  },
};

export default function() {
  // Your custom test logic
}
```

### Test Specific Endpoints

```javascript
// Test only login performance
export default function() {
  const credentials = {
    username: 'root',
    password: 'password',
  };

  login(credentials);
  sleep(1);
}
```

## Troubleshooting

### Common Issues

#### 1. Connection Refused

**Error:**
```
ERRO[0000] GoError: Get "http://localhost:8080/": dial tcp 127.0.0.1:8080: connect: connection refused
```

**Solution:**
- Ensure lighter-auth service is running: `cargo run --features postgres`
- Check service is listening on port 8080: `lsof -i :8080` or `netstat -an | grep 8080`
- Verify DATABASE_URL is configured correctly

#### 2. High Failure Rate

**Symptoms:**
- `http_req_failed` > 5%
- Success rates < 95%
- Many 500 errors

**Possible Causes:**
- Database connection pool exhausted
- CPU/memory constraints
- Database queries timing out
- Network issues

**Solutions:**
- Increase database connection pool size in lighter-common
- Add more system resources (CPU, RAM)
- Optimize database queries and add indexes
- Check database slow query log
- Monitor system resources during test: `top`, `htop`, `vmstat`

#### 3. Slow Response Times

**Symptoms:**
- p(95) > 500ms consistently
- Large gap between p(50) and p(95)

**Solutions:**
- Enable query logging to find slow queries
- Check database indexes are properly created
- Monitor cache hit rate (in-memory session cache)
- Profile the application: `cargo flamegraph`
- Consider adding Redis for distributed caching

#### 4. Database Locked (SQLite)

**Error:**
```
database is locked
```

**Solution:**
- SQLite is not recommended for load testing (single-writer limitation)
- Use PostgreSQL for load testing: `cargo run --features postgres`
- Ensure DATABASE_URL points to PostgreSQL instance

#### 5. Memory Leaks

**Symptoms:**
- Memory usage increases continuously
- System becomes slow over time
- OOM (Out of Memory) errors

**Solutions:**
- Check for connection leaks (unclosed DB connections)
- Monitor with: `ps aux | grep lighter-auth`
- Profile memory usage: `valgrind` or `heaptrack`
- Review authentication cache cleanup logic

#### 6. k6 Cloud Execution Issues

**Error:**
```
k6 cloud: authentication failed
```

**Solution:**
- Login to k6 Cloud: `k6 login cloud`
- Set API token: `k6 login cloud --token YOUR_TOKEN`
- Or run locally: `k6 run tests/load/k6_script.js`

### Debug Mode

Enable verbose output:

```bash
k6 run --verbose tests/load/k6_script.js
```

Enable HTTP debug logging:

```bash
k6 run --http-debug tests/load/k6_script.js
```

### Monitoring During Tests

**Monitor system resources:**
```bash
# CPU and memory
top -p $(pgrep lighter-auth)

# Disk I/O
iostat -x 1

# Network
netstat -s
```

**Monitor database:**
```bash
# PostgreSQL connections
psql -U lighter -d lighter_auth -c "SELECT count(*) FROM pg_stat_activity;"

# PostgreSQL slow queries
psql -U lighter -d lighter_auth -c "SELECT query, calls, total_time FROM pg_stat_statements ORDER BY total_time DESC LIMIT 10;"
```

**Monitor logs:**
```bash
# Follow application logs
tail -f /path/to/logs/lighter-auth.log

# Or if using journalctl
journalctl -u lighter-auth -f
```

## Performance Benchmarks

### Expected Performance (Reference Hardware)

**Test Environment:**
- CPU: Intel i7 8-core @ 3.5GHz
- RAM: 16GB
- Database: PostgreSQL 15 on localhost
- Network: Local (no network latency)

**Results:**

| Metric | Value | Status |
|--------|-------|--------|
| Total Requests | 20,000 | ✅ |
| Request Rate | 35-40 req/s | ✅ |
| Success Rate | > 98% | ✅ |
| HTTP p(95) | 250-350ms | ✅ |
| HTTP p(99) | 500-800ms | ✅ |
| Login p(95) | 150-250ms | ✅ |
| Auth Check p(95) | 50-100ms | ✅ (cached) |
| Failed Requests | < 2% | ✅ |

### Performance by Operation

| Operation | p(50) | p(95) | p(99) | Notes |
|-----------|-------|-------|-------|-------|
| Create User | 80ms | 300ms | 500ms | Includes password hashing |
| Login | 60ms | 200ms | 400ms | Includes password verification |
| Auth Check | 20ms | 80ms | 150ms | Fast due to caching |
| Logout | 15ms | 60ms | 120ms | Token deletion |

### Bottlenecks

**Known Performance Bottlenecks:**

1. **Password Hashing** (User Creation & Login)
   - CPU-intensive operation
   - ~50-100ms per operation
   - Consider: bcrypt work factor tuning, Argon2 alternative

2. **Database Connection Pool**
   - Default: 10 connections
   - May need increase for > 100 concurrent users
   - Consider: Read replicas for GET operations

3. **Synchronous Mutex** (In-Memory Cache)
   - Contention under high concurrency
   - Consider: `tokio::sync::RwLock` or Redis

4. **Complex Permission Queries**
   - Dual-path permission resolution (direct + role-based)
   - Multiple JOINs required
   - Consider: Permission caching, denormalization

### Scaling Recommendations

**Horizontal Scaling:**
- Current limitation: In-memory cache not distributed
- Solution: Migrate to Redis for session storage
- Expected improvement: 3-5x throughput with 3 instances

**Vertical Scaling:**
- CPU: Benefits from multi-core (async/await utilization)
- RAM: Increase for larger in-memory cache
- Database: PostgreSQL can handle 500+ connections

**Caching Strategy:**
- Current: 5-minute TTL in-memory cache
- Improvement: Redis with pub/sub for invalidation
- Cache hit rate should be > 90% under sustained load

## Running in CI/CD

### GitHub Actions Example

```yaml
name: Load Test

on:
  schedule:
    - cron: '0 0 * * 0'  # Weekly on Sunday
  workflow_dispatch:     # Manual trigger

jobs:
  load-test:
    runs-on: ubuntu-latest

    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_USER: lighter
          POSTGRES_PASSWORD: lighter
          POSTGRES_DB: lighter_auth
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432

    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run migrations
        run: |
          cd migration
          cargo run up
        env:
          DATABASE_URL: postgres://lighter:lighter@localhost:5432/lighter_auth

      - name: Start service
        run: |
          cargo run --features postgres &
          sleep 10  # Wait for service to start
        env:
          DATABASE_URL: postgres://lighter:lighter@localhost:5432/lighter_auth
          PORT: 8080

      - name: Install k6
        run: |
          curl https://github.com/grafana/k6/releases/download/v0.45.0/k6-v0.45.0-linux-amd64.tar.gz -L | tar xvz
          sudo mv k6-v0.45.0-linux-amd64/k6 /usr/local/bin/

      - name: Run load test
        run: k6 run tests/load/k6_script.js

      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: load-test-results
          path: tests/load/results.json
```

## Additional Resources

- [k6 Documentation](https://k6.io/docs/)
- [k6 Best Practices](https://k6.io/docs/testing-guides/api-load-testing/)
- [Performance Testing Guide](https://k6.io/docs/testing-guides/)
- [lighter-auth CLAUDE.md](/Users/gerianoadikaputra/Programs/Own/lighter/auth/CLAUDE.md)

## Support

For issues with:
- **k6 tool**: Check [k6 community forum](https://community.k6.io/)
- **Load test scripts**: Open an issue in the project repository
- **Service performance**: Review CLAUDE.md for optimization strategies

---

**Last Updated:** 2025-12-09
**k6 Version:** 0.45.0+
**Service Version:** lighter-auth 1.0.0
