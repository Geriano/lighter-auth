/**
 * Spike Test Configuration for lighter-auth
 *
 * This test simulates sudden traffic spikes to validate:
 * - Auto-scaling behavior
 * - Circuit breaker patterns
 * - Rate limiting effectiveness
 * - System recovery time
 *
 * Pattern: Multiple sharp spikes from baseline to peak load
 *
 * Duration: ~10 minutes
 * Max VUs: 200
 *
 * Use Cases:
 * - Marketing campaigns causing traffic surges
 * - DDoS attack simulation
 * - Flash sales or viral events
 * - Auto-scaler validation
 *
 * Usage:
 *   k6 run tests/load/spike-test.js
 */

// Import all functions from the main test script
export { default, setup, teardown, handleSummary } from './k6_script.js';

// Override options with spike pattern
export const options = {
  stages: [
    // Baseline load
    { duration: '1m', target: 20 },   // Normal baseline

    // First spike
    { duration: '10s', target: 150 }, // Rapid spike to 150 users
    { duration: '1m', target: 150 },  // Hold spike
    { duration: '10s', target: 20 },  // Drop back to baseline

    // Recovery period
    { duration: '1m', target: 20 },   // Measure recovery

    // Second spike (even larger)
    { duration: '10s', target: 200 }, // Extreme spike to 200 users
    { duration: '1m', target: 200 },  // Hold spike
    { duration: '10s', target: 20 },  // Drop back to baseline

    // Final recovery
    { duration: '1m', target: 20 },   // Measure recovery
    { duration: '30s', target: 0 },   // Ramp down
  ],

  thresholds: {
    // Response times may degrade during spikes
    'http_req_duration': [
      'p(95)<1500',    // 95% under 1.5 seconds (allow for spike impact)
      'p(99)<3000',    // 99% under 3 seconds
    ],

    // Error rates may increase during spikes
    'http_req_failed': ['rate<0.15'],  // Allow up to 15% errors during spikes

    // Success rates should recover quickly
    'login_success_rate': ['rate>0.85'],
    'create_user_success_rate': ['rate>0.85'],
    'auth_user_success_rate': ['rate>0.90'],  // Cached, should stay high
    'logout_success_rate': ['rate>0.90'],

    // Throughput will vary significantly
    'http_reqs': ['rate>5'],  // Just maintain minimum throughput

    // Operation durations
    'login_duration': ['p(95)<1000'],
    'create_user_duration': ['p(95)<1500'],
    'auth_user_duration': ['p(95)<500'],
    'logout_duration': ['p(95)<500'],
  },

  noConnectionReuse: false,
  userAgent: 'K6SpikeTest/1.0',

  tags: {
    testType: 'spike',
    service: 'lighter-auth',
  },
};

// Notes for interpreting spike test results:
//
// 1. Look for:
//    - How quickly error rates increase during spike
//    - How long it takes to recover after spike ends
//    - Whether errors occur in first spike vs second spike
//    - If cache helps during spikes (auth_user should be fast)
//
// 2. Warning signs:
//    - Errors continue after spike ends (slow recovery)
//    - Error rate increases linearly with load (no graceful degradation)
//    - Memory/connection leaks (check system resources)
//    - Database connection pool exhaustion
//
// 3. Expected behavior:
//    - Some errors during spike peaks (acceptable)
//    - Quick recovery when load drops (< 30 seconds)
//    - Cached operations remain fast (auth checks)
//    - Database queries may slow but should not fail
