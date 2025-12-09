/**
 * Soak Test (Endurance Test) Configuration for lighter-auth
 *
 * This test validates system stability over extended periods to detect:
 * - Memory leaks
 * - Connection pool exhaustion
 * - Database connection leaks
 * - Cache corruption
 * - Performance degradation over time
 * - Resource exhaustion
 *
 * Pattern: Sustained moderate load for extended duration
 *
 * Duration: 2 hours (can be adjusted)
 * Max VUs: 50 (moderate constant load)
 *
 * WARNING: This test runs for 2+ hours. Ensure you have:
 * - Monitoring in place
 * - Sufficient system resources
 * - Database maintenance disabled during test
 *
 * Usage:
 *   k6 run tests/load/soak-test.js
 *
 * Quick version (30 minutes):
 *   k6 run tests/load/soak-test.js -e DURATION=30m
 */

// Import all functions from the main test script
export { default, setup, teardown, handleSummary } from './k6_script.js';

// Get duration from environment variable or use default
const SOAK_DURATION = __ENV.DURATION || '2h';

// Override options with soak test pattern
export const options = {
  stages: [
    { duration: '5m', target: 50 },     // Ramp up to moderate load
    { duration: SOAK_DURATION, target: 50 },  // Hold for extended period
    { duration: '5m', target: 0 },      // Ramp down
  ],

  thresholds: {
    // Stricter thresholds - performance should be consistent
    'http_req_duration': [
      'p(95)<500',     // Should stay under 500ms throughout
      'p(99)<1000',    // Should stay under 1000ms throughout
    ],

    // Very low error rate over time
    'http_req_failed': ['rate<0.02'],  // Less than 2% errors

    // High success rates throughout
    'login_success_rate': ['rate>0.98'],
    'create_user_success_rate': ['rate>0.98'],
    'auth_user_success_rate': ['rate>0.98'],
    'logout_success_rate': ['rate>0.98'],

    // Consistent throughput
    'http_reqs': ['rate>8'],  // At least 8 req/s sustained

    // Operation durations should not degrade
    'login_duration': ['p(95)<300'],
    'create_user_duration': ['p(95)<500'],
    'auth_user_duration': ['p(95)<200'],
    'logout_duration': ['p(95)<200'],
  },

  noConnectionReuse: false,
  userAgent: 'K6SoakTest/1.0',

  tags: {
    testType: 'soak',
    service: 'lighter-auth',
    duration: SOAK_DURATION,
  },
};

// Additional monitoring recommendations for soak tests:
//
// 1. System Metrics to Monitor:
//    - Memory usage: `ps aux | grep lighter-auth` every 5 minutes
//    - CPU usage: Should stay relatively constant
//    - Disk I/O: Check for unexpected writes
//    - Network connections: `netstat -an | grep :8080 | wc -l`
//
// 2. Database Metrics:
//    - Active connections: Should stay within pool limits
//    - Slow queries: Should not increase over time
//    - Table sizes: Check for unexpected growth
//    - Lock waits: Should remain minimal
//
// 3. Application Metrics:
//    - Cache hit rate: Should stay high (>90%)
//    - Token count: Check for token cleanup
//    - Error logs: Watch for recurring errors
//    - Response time trend: Should stay flat
//
// 4. What to Look For:
//    - Gradual performance degradation (indicates leak)
//    - Sudden drops in performance (indicates threshold)
//    - Increasing error rates over time
//    - Memory usage trending upward
//    - Database connection pool exhaustion
//
// 5. PostgreSQL Monitoring Queries:
//    ```sql
//    -- Active connections
//    SELECT count(*) FROM pg_stat_activity WHERE state = 'active';
//
//    -- Table sizes
//    SELECT pg_size_pretty(pg_total_relation_size('users'));
//    SELECT pg_size_pretty(pg_total_relation_size('tokens'));
//
//    -- Slow queries
//    SELECT query, calls, total_time, mean_time
//    FROM pg_stat_statements
//    ORDER BY mean_time DESC LIMIT 10;
//
//    -- Locks
//    SELECT * FROM pg_locks WHERE NOT granted;
//    ```
//
// 6. Expected Results:
//    - Flat performance curves throughout test
//    - Consistent memory usage (no growth)
//    - Stable database connection count
//    - No errors or very minimal errors
//    - Fast recovery after ramp-down
//
// 7. Failure Indicators:
//    - Memory usage increasing linearly
//    - Response times increasing over time
//    - Error rates climbing
//    - Database connection leaks
//    - Token table growing unbounded
