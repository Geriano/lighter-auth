# Example K6 Load Test Output

This document shows example output from running the k6 load tests to help you understand what to expect.

## Quick Test Output Example

```
          /\      |‾‾| /‾‾/   /‾‾/
     /\  /  \     |  |/  /   /  /
    /  \/    \    |     (   /   ‾‾\
   /          \   |  |\  \ |  (‾)  |
  / __________ \  |__| \__\ \_____/ .io

  execution: local
    script: tests/load/quick-test.js
    output: -

  scenarios: (100.00%) 1 scenario, 10 max VUs, 2m30s max duration (incl. graceful stop):
           * default: Up to 10 looping VUs for 2m0s over 3 stages (gracefulRampDown: 30s, gracefulStop: 30s)


================================================================================
K6 Load Test for lighter-auth Service
================================================================================
Base URL: http://localhost:8080
Test Type: Load Test
Test Duration: ~2 minutes (quick test)
Max VUs: 10
================================================================================

Service health check passed. Starting load test...

running (2m02.5s), 00/10 VUs, 185 complete and 0 interrupted iterations
default ✓ [======================================] 00/10 VUs  2m0s

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

     auth_user_duration............: avg=42.15ms  min=15.23ms med=36.78ms max=198.45ms p(90)=68.9ms  p(95)=89.23ms ✓
     auth_user_success_rate........: 99.45% ✓ 184       ✗ 1
     checks.........................: 99.59% ✓ 1478      ✗ 6
     create_user_duration..........: avg=156.78ms min=78.34ms med=142.56ms max=542.34ms p(90)=245.6ms p(95)=298.45ms ✓
     create_user_success_rate......: 98.91% ✓ 183       ✗ 2
     data_received..................: 389 kB 3.2 kB/s
     data_sent......................: 215 kB 1.8 kB/s
     failed_operations..............: 9      0.07/s
     http_req_blocked...............: avg=85.67µs  min=2µs     med=5µs     max=8.45ms   p(90)=8µs     p(95)=12µs
     http_req_connecting............: avg=42.34µs  min=0s      med=0s      max=4.23ms   p(90)=0s      p(95)=0s
     http_req_duration..............: avg=89.45ms  min=15.23ms med=78.56ms max=542.34ms p(90)=156.7ms p(95)=198.45ms
       { expected_response:true }...: avg=88.92ms  min=15.23ms med=78.34ms max=542.34ms p(90)=154.3ms p(95)=196.23ms
     ✓ { p(95)<800 }.................: true
     ✓ { p(99)<1500 }................: true
     http_req_failed................: 1.21%  ✓ 9         ✗ 731
     ✓ { rate<0.10 }.................: true
     http_req_receiving.............: avg=234µs    min=28µs    med=156µs   max=2.34ms   p(90)=456µs   p(95)=678µs
     http_req_sending...............: avg=123µs    min=15µs    med=89µs    max=1.23ms   p(90)=234µs   p(95)=345µs
     http_req_tls_handshaking.......: avg=0s       min=0s      med=0s      max=0s       p(90)=0s      p(95)=0s
     http_req_waiting...............: avg=89.09ms  min=15.12ms med=78.34ms max=541.98ms p(90)=155.8ms p(95)=197.67ms
     http_reqs......................: 740    6.08/s ✓
     ✓ { rate>5 }....................: true
     iteration_duration.............: avg=7.62s    min=6.45s   med=7.56s   max=9.87s    p(90)=8.45s   p(95)=8.98s
     iterations.....................: 185    1.52/s
     login_duration.................: avg=78.45ms  min=34.56ms med=67.89ms max=298.76ms p(90)=134.5ms p(95)=167.8ms ✓
     login_success_rate.............: 99.45% ✓ 184       ✗ 1
     ✓ { rate>0.90 }.................: true
     logout_duration................: avg=38.67ms  min=12.34ms med=32.45ms max=156.78ms p(90)=67.8ms  p(95)=89.12ms ✓
     logout_success_rate............: 98.91% ✓ 183       ✗ 2
     ✓ { rate>0.90 }.................: true
     total_operations...............: 740    6.08/s
     vus............................: 0      min=0       max=10
     vus_max........................: 10     min=10      max=10

================================================================================
Load Test Complete
================================================================================

Check the detailed results above for:
  - HTTP request duration percentiles (p95, p99)
  - Success rates for each operation
  - Total operations and throughput
  - Failed operations count

Refer to the thresholds section to see if the test passed.
```

## Standard Load Test Output Example

```
          /\      |‾‾| /‾‾/   /‾‾/
     /\  /  \     |  |/  /   /  /
    /  \/    \    |     (   /   ‾‾\
   /          \   |  |\  \ |  (‾)  |
  / __________ \  |__| \__\ \_____/ .io

  execution: local
    script: tests/load/k6_script.js
    output: -

  scenarios: (100.00%) 1 scenario, 100 max VUs, 9m30s max duration (incl. graceful stop):
           * default: Up to 100 looping VUs for 9m0s over 5 stages (gracefulRampDown: 30s)


================================================================================
K6 Load Test for lighter-auth Service
================================================================================
Base URL: http://localhost:8080
Test Type: Load Test
Test Duration: ~9 minutes (including ramp up/down)
Max VUs: 100
================================================================================

Service health check passed. Starting load test...

running (9m02.3s), 000/100 VUs, 4523 complete and 0 interrupted iterations
default ✓ [======================================] 000/100 VUs  9m0s

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

     auth_user_duration............: avg=45.89ms  min=12.45ms med=38.67ms max=289.45ms p(90)=82.3ms  p(95)=108.9ms
     auth_user_success_rate........: 98.89% ✓ 4473      ✗ 50
     checks.........................: 99.31% ✓ 35982     ✗ 250
     create_user_duration..........: avg=134.56ms min=56.78ms med=112.34ms max=987.65ms p(90)=256.8ms p(95)=367.9ms
     create_user_success_rate......: 97.98% ✓ 4432      ✗ 91
     data_received..................: 18 MB  33 kB/s
     data_sent......................: 9.8 MB 18 kB/s
     failed_operations..............: 432    0.79/s
     http_req_blocked...............: avg=2.45ms   min=1µs     med=4µs     max=145.67ms p(90)=6µs     p(95)=8µs
     http_req_connecting............: avg=1.34ms   min=0s      med=0s      max=123.45ms p(90)=0s      p(95)=0s
     http_req_duration..............: avg=92.78ms  min=12.45ms med=76.89ms max=987.65ms p(90)=167.8ms p(95)=256.7ms
       { expected_response:true }...: avg=92.34ms  min=12.45ms med=76.56ms max=945.32ms p(90)=165.4ms p(95)=254.3ms
     ✓ { p(95)<500 }.................: true
     ✓ { p(99)<1000 }................: true
     http_req_failed................: 2.38%  ✓ 432       ✗ 17660
     ✓ { rate<0.05 }.................: true
     http_req_receiving.............: avg=245µs    min=24µs    med=145µs   max=15.67ms  p(90)=478µs   p(95)=689µs
     http_req_sending...............: avg=134µs    min=13µs    med=78µs    max=8.45ms   p(90)=245µs   p(95)=367µs
     http_req_tls_handshaking.......: avg=0s       min=0s      med=0s      max=0s       p(90)=0s      p(95)=0s
     http_req_waiting...............: avg=92.40ms  min=12.32ms med=76.67ms max=985.43ms p(90)=167.2ms p(95)=255.9ms
     http_reqs......................: 18092  33.28/s ✓
     ✓ { rate>10 }....................: true
     iteration_duration.............: avg=7.58s    min=6.12s   med=7.48s   max=10.98s   p(90)=8.67s   p(95)=9.23s
     iterations.....................: 4523   8.32/s
     login_duration.................: avg=71.23ms  min=28.45ms med=61.78ms max=645.32ms p(90)=134.5ms p(95)=189.6ms
     login_success_rate.............: 98.67% ✓ 4463      ✗ 60
     ✓ { rate>0.95 }.................: true
     logout_duration................: avg=36.78ms  min=9.87ms  med=30.45ms max=287.65ms p(90)=72.3ms  p(95)=94.5ms
     logout_success_rate............: 99.27% ✓ 4490      ✗ 33
     ✓ { rate>0.95 }.................: true
     total_operations...............: 18092  33.28/s
     vus............................: 0      min=0       max=100
     vus_max........................: 100    min=100     max=100


================================================================================
Load Test Results Summary
================================================================================

Test Duration: 542.31s

HTTP Metrics:
  Total Requests: 18092
  Request Rate: 33.28 req/s
  Failed Requests: 2.38%
  Request Duration (p95): 256.70ms
  Request Duration (p99): 543.21ms

Operation Success Rates:
  User Creation: 97.98%
  Login: 98.67%
  Auth Check: 98.89%
  Logout: 99.27%

Operation Durations (p95):
  User Creation: 367.90ms
  Login: 189.60ms
  Auth Check: 108.90ms
  Logout: 94.50ms

Checks:
  Passed: 35982
  Failed: 250
  Pass Rate: 99.31%

================================================================================
```

## Stress Test Output Example (with some failures)

```
running (15m04.2s), 000/250 VUs, 8234 complete and 0 interrupted iterations
default ✓ [======================================] 000/250 VUs  15m0s

     ✓ user created successfully
     ✓ user response has id
     ✗ login successful
      ↳  95% — ✓ 7845 / ✗ 389
     ✗ login response has token
      ↳  95% — ✓ 7845 / ✗ 389
     ✗ authenticated user retrieved
      ↳  92% — ✓ 7589 / ✗ 645
     ✗ logout successful
      ↳  93% — ✓ 7658 / ✗ 576

     auth_user_duration............: avg=289.45ms min=15.67ms med=234.56ms max=4567.89ms p(90)=678.9ms p(95)=987.6ms
     auth_user_success_rate........: 92.17% ✓ 7589      ✗ 645
     checks.........................: 95.23% ✓ 62784     ✗ 3152
     create_user_duration..........: avg=456.78ms min=87.65ms med=389.45ms max=5678.90ms p(90)=987.6ms p(95)=1456.7ms
     create_user_success_rate......: 96.78% ✓ 7970      ✗ 264
     http_req_duration..............: avg=345.67ms min=15.67ms med=289.45ms max=5678.90ms p(90)=876.5ms p(95)=1234.5ms
     ✓ { p(95)<2000 }................: true
     ✓ { p(99)<5000 }................: true
     http_req_failed................: 12.45% ✓ 4098      ✗ 28826
     ✓ { rate<0.20 }.................: true
     login_success_rate.............: 95.27% ✓ 7845      ✗ 389
     ✓ { rate>0.80 }.................: true
     logout_success_rate............: 93.00% ✓ 7658      ✗ 576
     ✓ { rate>0.85 }.................: true
     total_operations...............: 32936  36.45/s

⚠️ Some thresholds have failed

================================================================================
Load Test Results Summary
================================================================================

Test Duration: 904.23s

HTTP Metrics:
  Total Requests: 32936
  Request Rate: 36.45 req/s
  Failed Requests: 12.45%  ⚠️ HIGH
  Request Duration (p95): 1234.50ms  ⚠️ DEGRADED
  Request Duration (p99): 3456.78ms  ⚠️ SLOW

Operation Success Rates:
  User Creation: 96.78%  ✓
  Login: 95.27%  ✓
  Auth Check: 92.17%  ⚠️ BELOW TARGET
  Logout: 93.00%  ✓

⚠️ WARNINGS:
  - High error rate detected (12.45%)
  - Auth check success rate below 95%
  - Response times degraded under stress
  - System showing signs of strain at 250 concurrent users

RECOMMENDATIONS:
  1. Check database connection pool settings
  2. Monitor memory usage for leaks
  3. Review slow query logs
  4. Consider horizontal scaling beyond 200 users
```

## Interpreting the Output

### Key Sections

1. **Header**: Shows test configuration and duration
2. **Checks**: Individual validation pass/fail counts
3. **Metrics**: Detailed performance statistics
4. **Thresholds**: Pass/fail status (✓ or ✗)
5. **Summary**: Human-readable results overview

### Important Metrics

- **http_req_duration p(95)**: 95% of requests faster than this
- **http_req_failed**: Percentage of failed requests
- **Success rates**: Per-operation success percentages
- **http_reqs**: Total throughput (requests per second)
- **iteration_duration**: Full user journey time

### Threshold Indicators

- ✓ = Threshold passed (green in terminal)
- ✗ = Threshold failed (red in terminal)
- No checkmark = Informational metric

### What to Look For

**Healthy System:**
- All thresholds passing
- Success rates > 98%
- Low variance in response times
- Stable throughout test

**System Under Stress:**
- Some thresholds failing
- Success rates 90-95%
- High variance in response times
- Degradation over time

**Critical Issues:**
- Multiple threshold failures
- Success rates < 90%
- Very high response times
- Errors increasing over time

## Saved Results

Results are also saved to JSON for analysis:

```bash
# View saved results
cat tests/load/results.json | jq '.metrics.http_req_duration.values'

# Extract key metrics
cat tests/load/results.json | jq '{
  total_requests: .metrics.http_reqs.values.count,
  error_rate: .metrics.http_req_failed.values.rate,
  p95: .metrics.http_req_duration.values["p(95)"],
  p99: .metrics.http_req_duration.values["p(99)"]
}'
```

---

**Note:** Actual output will vary based on system performance, load, and database configuration. Use these examples as references for understanding the format and metrics.
