import http from "k6/http";
import { check, sleep } from "k6";
import { Rate, Counter, Trend } from "k6/metrics";
import { randomString } from "https://jslib.k6.io/k6-utils/1.2.0/index.js";

// Configuration
const BASE_URL = __ENV.BASE_URL || "http://localhost:8080";

// Custom metrics
const loginSuccessRate = new Rate("login_success_rate");
const createUserSuccessRate = new Rate("create_user_success_rate");
const authUserSuccessRate = new Rate("auth_user_success_rate");
const logoutSuccessRate = new Rate("logout_success_rate");

const loginDuration = new Trend("login_duration");
const createUserDuration = new Trend("create_user_duration");
const authUserDuration = new Trend("auth_user_duration");
const logoutDuration = new Trend("logout_duration");

const totalOperations = new Counter("total_operations");
const failedOperations = new Counter("failed_operations");

// Load test configuration
export const options = {
  stages: [
    { duration: "1m", target: 50 }, // Ramp up to 50 users over 1 minute
    { duration: "3m", target: 50 }, // Stay at 50 users for 3 minutes
    { duration: "1m", target: 100 }, // Ramp up to 100 users over 1 minute
    { duration: "3m", target: 100 }, // Stay at 100 users for 3 minutes
    { duration: "1m", target: 0 }, // Ramp down to 0 users over 1 minute
  ],

  thresholds: {
    // HTTP request duration thresholds
    http_req_duration: [
      "p(95)<500", // 95% of requests should be below 500ms
      "p(99)<1000", // 99% of requests should be below 1000ms
    ],

    // Overall error rate should be less than 5%
    http_req_failed: ["rate<0.05"],

    // Custom metric thresholds
    login_success_rate: ["rate>0.95"], // Login success rate > 95%
    create_user_success_rate: ["rate>0.95"], // User creation success rate > 95%
    auth_user_success_rate: ["rate>0.95"], // Auth check success rate > 95%
    logout_success_rate: ["rate>0.95"], // Logout success rate > 95%

    // Minimum throughput
    http_reqs: ["rate>10"], // At least 10 requests per second

    // Specific operation duration thresholds
    login_duration: ["p(95)<300"], // Login should be fast
    create_user_duration: ["p(95)<500"], // User creation moderate
    auth_user_duration: ["p(95)<200"], // Auth check should be very fast (cached)
    logout_duration: ["p(95)<200"], // Logout should be fast
  },

  // Additional configuration
  noConnectionReuse: false, // Reuse connections for better performance
  userAgent: "K6LoadTest/1.0",

  // Tags for better reporting
  tags: {
    testType: "load",
    service: "lighter-auth",
  },
};

/**
 * Generate a unique username based on VU and iteration
 */
function generateUsername() {
  return `user_${__VU}_${__ITER}_${randomString(6)}`;
}

/**
 * Generate a unique email based on VU and iteration
 */
function generateEmail() {
  return `user_${__VU}_${__ITER}_${randomString(6)}@loadtest.local`;
}

/**
 * Generate a secure password
 */
function generatePassword() {
  // Password requirements: min 8 chars, uppercase, lowercase, number, special char
  return `LoadTest${randomString(8)}@123`;
}

/**
 * Create a new user
 */
function createUser() {
  const username = generateUsername();
  const email = generateEmail();
  const password = generatePassword();

  const payload = JSON.stringify({
    name: `Load Test User ${__VU}_${__ITER}`,
    email: email,
    username: username,
    password: password,
    passwordConfirmation: password,
    profilePhotoId: null,
    permissions: [],
    roles: [],
  });

  const params = {
    headers: {
      "Content-Type": "application/json",
    },
    tags: { operation: "create_user" },
  };

  const startTime = new Date();
  const response = http.post(`${BASE_URL}/v1/user`, payload, params);
  const duration = new Date() - startTime;

  totalOperations.add(1);
  createUserDuration.add(duration);

  const success = check(response, {
    "user created successfully": (r) => r.status === 200,
    "user response has id": (r) => {
      if (r.status === 200) {
        try {
          const body = JSON.parse(r.body);
          return body.id !== undefined;
        } catch (e) {
          return false;
        }
      }
      return false;
    },
  });

  createUserSuccessRate.add(success);

  if (!success) {
    failedOperations.add(1);
    console.error(
      `Failed to create user: ${response.status} - ${response.body}`,
    );
    return null;
  }

  const userData = JSON.parse(response.body);

  return {
    id: userData.id,
    username: username,
    email: email,
    password: password,
  };
}

/**
 * Login with user credentials
 */
function login(credentials) {
  const payload = JSON.stringify({
    emailOrUsername: credentials.username,
    password: credentials.password,
  });

  const params = {
    headers: {
      "Content-Type": "application/json",
    },
    tags: { operation: "login" },
  };

  const startTime = new Date();
  const response = http.post(`${BASE_URL}/login`, payload, params);
  const duration = new Date() - startTime;

  totalOperations.add(1);
  loginDuration.add(duration);

  const success = check(response, {
    "login successful": (r) => r.status === 200,
    "login response has token": (r) => {
      if (r.status === 200) {
        try {
          const body = JSON.parse(r.body);
          return body.token !== undefined && body.token !== "";
        } catch (e) {
          return false;
        }
      }
      return false;
    },
    "login response has user data": (r) => {
      if (r.status === 200) {
        try {
          const body = JSON.parse(r.body);
          return body.user !== undefined && body.user.id !== undefined;
        } catch (e) {
          return false;
        }
      }
      return false;
    },
  });

  loginSuccessRate.add(success);

  if (!success) {
    failedOperations.add(1);
    console.error(`Failed to login: ${response.status} - ${response.body}`);
    return null;
  }

  const loginData = JSON.parse(response.body);
  return loginData.token;
}

/**
 * Get authenticated user information
 */
function getAuthenticatedUser(token) {
  const params = {
    headers: {
      Authorization: `Bearer ${token}`,
    },
    tags: { operation: "get_authenticated_user" },
  };

  const startTime = new Date();
  const response = http.get(`${BASE_URL}/user`, params);
  const duration = new Date() - startTime;

  totalOperations.add(1);
  authUserDuration.add(duration);

  const success = check(response, {
    "authenticated user retrieved": (r) => r.status === 200,
    "authenticated user has data": (r) => {
      if (r.status === 200) {
        try {
          const body = JSON.parse(r.body);
          return body.user.id !== undefined && body.user.email !== undefined;
        } catch (e) {
          return false;
        }
      }
      return false;
    },
  });

  authUserSuccessRate.add(success);

  if (!success) {
    failedOperations.add(1);
    console.error(
      `Failed to get authenticated user: ${response.status} - ${response.body}`,
    );
  }

  return success;
}

/**
 * Logout user
 */
function logout(token) {
  const params = {
    headers: {
      Authorization: `Bearer ${token}`,
    },
    tags: { operation: "logout" },
  };

  const startTime = new Date();
  const response = http.del(`${BASE_URL}/logout`, null, params);
  const duration = new Date() - startTime;

  totalOperations.add(1);
  logoutDuration.add(duration);

  const success = check(response, {
    "logout successful": (r) => r.status === 200,
  });

  logoutSuccessRate.add(success);

  if (!success) {
    failedOperations.add(1);
    console.error(`Failed to logout: ${response.status} - ${response.body}`);
  }

  return success;
}

/**
 * Main test scenario
 * This simulates a complete user lifecycle:
 * 1. Create user
 * 2. Login
 * 3. Access authenticated endpoint (simulates actual usage)
 * 4. Logout
 */
export default function () {
  // 1. Create a new user
  const user = createUser();

  if (!user) {
    console.error("Failed to create user, skipping iteration");
    sleep(2);
    return;
  }

  // Think time between operations
  sleep(1);

  // 2. Login with the created user
  const token = login(user);

  if (!token) {
    console.error("Failed to login, skipping iteration");
    sleep(2);
    return;
  }

  // Think time after login
  sleep(1);

  // 3. Get authenticated user information (simulates actual usage)
  // This tests the auth middleware and caching
  const authSuccess = getAuthenticatedUser(token);

  if (!authSuccess) {
    console.error("Failed to get authenticated user");
  }

  // Think time between operations
  sleep(1);

  // 4. Logout
  logout(token);

  // Think time before next iteration (simulates user browsing between actions)
  sleep(2);
}

/**
 * Setup function - runs once at the start of the test
 */
export function setup() {
  console.log(`\n${"=".repeat(80)}`);
  console.log("K6 Load Test for lighter-auth Service");
  console.log(`${"=".repeat(80)}`);
  console.log(`Base URL: ${BASE_URL}`);
  console.log(`Test Type: Load Test`);
  console.log(`Test Duration: ~9 minutes (including ramp up/down)`);
  console.log(`Max VUs: 100`);
  console.log(`${"=".repeat(80)}\n`);

  // Verify the service is accessible
  const healthCheck = http.get(`${BASE_URL}/`);

  if (healthCheck.status !== 200) {
    console.error(`ERROR: Service is not accessible at ${BASE_URL}`);
    console.error(`Status: ${healthCheck.status}`);
    console.error(`Body: ${healthCheck.body}`);
    throw new Error("Service health check failed");
  }

  console.log("Service health check passed. Starting load test...\n");
}

/**
 * Teardown function - runs once at the end of the test
 */
export function teardown(data) {
  console.log(`\n${"=".repeat(80)}`);
  console.log("Load Test Complete");
  console.log(`${"=".repeat(80)}\n`);
  console.log("Check the detailed results above for:");
  console.log("  - HTTP request duration percentiles (p95, p99)");
  console.log("  - Success rates for each operation");
  console.log("  - Total operations and throughput");
  console.log("  - Failed operations count");
  console.log("\nRefer to the thresholds section to see if the test passed.\n");
}

/**
 * Custom summary handler for better reporting
 */
export function handleSummary(data) {
  return {
    stdout: textSummary(data, { indent: " ", enableColors: true }),
    "/Users/gerianoadikaputra/Programs/Own/lighter/auth/tests/load/results.json":
      JSON.stringify(data),
  };
}

/**
 * Text summary helper
 */
function textSummary(data, options) {
  const indent = options.indent || "";
  const enableColors = options.enableColors || false;

  let summary = "\n";
  summary += `${indent}${"=".repeat(80)}\n`;
  summary += `${indent}Load Test Results Summary\n`;
  summary += `${indent}${"=".repeat(80)}\n\n`;

  // Test duration
  summary += `${indent}Test Duration: ${(data.state.testRunDurationMs / 1000).toFixed(2)}s\n\n`;

  // HTTP metrics
  summary += `${indent}HTTP Metrics:\n`;
  summary += `${indent}  Total Requests: ${data.metrics.http_reqs.values.count}\n`;
  summary += `${indent}  Request Rate: ${data.metrics.http_reqs.values.rate.toFixed(2)} req/s\n`;
  summary += `${indent}  Failed Requests: ${(data.metrics.http_req_failed.values.rate * 100).toFixed(2)}%\n`;
  summary += `${indent}  Request Duration (p95): ${data.metrics.http_req_duration.values["p(95)"].toFixed(2)}ms\n`;
  summary += `${indent}  Request Duration (p99): ${data.metrics.http_req_duration.values["p(99)"].toFixed(2)}ms\n\n`;

  // Custom metrics
  summary += `${indent}Operation Success Rates:\n`;
  summary += `${indent}  User Creation: ${(data.metrics.create_user_success_rate.values.rate * 100).toFixed(2)}%\n`;
  summary += `${indent}  Login: ${(data.metrics.login_success_rate.values.rate * 100).toFixed(2)}%\n`;
  summary += `${indent}  Auth Check: ${(data.metrics.auth_user_success_rate.values.rate * 100).toFixed(2)}%\n`;
  summary += `${indent}  Logout: ${(data.metrics.logout_success_rate.values.rate * 100).toFixed(2)}%\n\n`;

  // Operation durations
  summary += `${indent}Operation Durations (p95):\n`;
  summary += `${indent}  User Creation: ${data.metrics.create_user_duration.values["p(95)"].toFixed(2)}ms\n`;
  summary += `${indent}  Login: ${data.metrics.login_duration.values["p(95)"].toFixed(2)}ms\n`;
  summary += `${indent}  Auth Check: ${data.metrics.auth_user_duration.values["p(95)"].toFixed(2)}ms\n`;
  summary += `${indent}  Logout: ${data.metrics.logout_duration.values["p(95)"].toFixed(2)}ms\n\n`;

  // Checks
  summary += `${indent}Checks:\n`;
  summary += `${indent}  Passed: ${data.metrics.checks.values.passes}\n`;
  summary += `${indent}  Failed: ${data.metrics.checks.values.fails}\n`;
  summary += `${indent}  Pass Rate: ${(data.metrics.checks.values.rate * 100).toFixed(2)}%\n\n`;

  summary += `${indent}${"=".repeat(80)}\n\n`;

  return summary;
}
