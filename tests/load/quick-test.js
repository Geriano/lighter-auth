/**
 * Quick Load Test Configuration for lighter-auth
 *
 * This is a shorter version of the main load test, suitable for:
 * - Development testing
 * - Quick smoke tests
 * - CI/CD pipeline checks
 *
 * Duration: ~2 minutes (vs 9 minutes for full test)
 * Max VUs: 20 (vs 100 for full test)
 *
 * Usage:
 *   k6 run tests/load/quick-test.js
 */

// Import all functions from the main test script
export { default, setup, teardown, handleSummary } from './k6_script.js';

// Override options with a quicker load profile
export const options = {
  stages: [
    { duration: '30s', target: 10 },  // Ramp up to 10 users
    { duration: '1m', target: 10 },   // Hold at 10 users
    { duration: '30s', target: 0 },   // Ramp down
  ],

  thresholds: {
    // Relaxed thresholds for quick testing
    'http_req_duration': ['p(95)<800', 'p(99)<1500'],
    'http_req_failed': ['rate<0.10'],  // Allow 10% error rate
    'login_success_rate': ['rate>0.90'],
    'create_user_success_rate': ['rate>0.90'],
    'auth_user_success_rate': ['rate>0.90'],
    'logout_success_rate': ['rate>0.90'],
    'http_reqs': ['rate>5'],  // Minimum 5 req/s
  },

  noConnectionReuse: false,
  userAgent: 'K6QuickTest/1.0',

  tags: {
    testType: 'smoke',
    service: 'lighter-auth',
  },
};
