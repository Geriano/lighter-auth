/**
 * Stress Test Configuration for lighter-auth
 *
 * This test pushes the system beyond normal operating capacity to:
 * - Identify breaking points
 * - Test system recovery
 * - Validate error handling under extreme load
 * - Identify resource leaks
 *
 * Duration: ~15 minutes
 * Max VUs: 250 (2.5x normal load)
 *
 * WARNING: This test may cause service degradation or failure.
 * Only run in dedicated test environments!
 *
 * Usage:
 *   k6 run tests/load/stress-test.js
 */

// Import all functions from the main test script
export { default, setup, teardown, handleSummary } from './k6_script.js';

// Override options with an aggressive load profile
export const options = {
  stages: [
    { duration: '2m', target: 50 },   // Warm up to 50 users
    { duration: '2m', target: 100 },  // Ramp to normal load
    { duration: '3m', target: 150 },  // Increase to 1.5x normal
    { duration: '3m', target: 200 },  // Push to 2x normal
    { duration: '3m', target: 250 },  // Maximum stress at 2.5x
    { duration: '2m', target: 0 },    // Cool down and recovery
  ],

  thresholds: {
    // More lenient thresholds - we expect degradation
    'http_req_duration': [
      'p(95)<2000',    // 95% under 2 seconds (vs 500ms normal)
      'p(99)<5000',    // 99% under 5 seconds (vs 1000ms normal)
    ],

    // Allow higher error rates under stress
    'http_req_failed': ['rate<0.20'],  // Allow up to 20% errors

    // Operation-specific thresholds
    'login_success_rate': ['rate>0.80'],        // 80% minimum
    'create_user_success_rate': ['rate>0.80'],  // 80% minimum
    'auth_user_success_rate': ['rate>0.85'],    // 85% (should be cached)
    'logout_success_rate': ['rate>0.85'],       // 85% minimum

    // Throughput - just need to maintain some level
    'http_reqs': ['rate>5'],  // At least 5 req/s even under stress

    // Individual operation durations
    'login_duration': ['p(95)<1500'],       // Login under stress
    'create_user_duration': ['p(95)<2000'], // Creation under stress
    'auth_user_duration': ['p(95)<800'],    // Auth check under stress
    'logout_duration': ['p(95)<800'],       // Logout under stress
  },

  // More aggressive HTTP settings
  noConnectionReuse: false,
  userAgent: 'K6StressTest/1.0',

  // Batch multiple requests together
  batch: 10,
  batchPerHost: 5,

  // Disable some checks for performance
  discardResponseBodies: false,  // Keep false to validate responses

  tags: {
    testType: 'stress',
    service: 'lighter-auth',
  },
};

// Note: The test scenario (default function) is imported from k6_script.js
// You can override it here if you want different behavior under stress
