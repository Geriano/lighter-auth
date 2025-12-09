# SQL Injection Security Tests

Comprehensive security tests verifying that the lighter-auth service is protected against SQL injection attacks through SeaORM's parameterized queries.

## Test Coverage

### User Creation Tests
- **Email field injection**: `test'; DROP TABLE users; --@example.com`
- **Username field injection**: `admin' OR '1'='1`
- **Name field injection**: `'; DELETE FROM users WHERE '1'='1'; --`

### Login Tests
- **Email/username bypass**: `admin' OR '1'='1' --`
- **Password field injection**: `' OR 1=1 --`
- **Combined field injection**: Both email and password with SQL

### User Update Tests
- **Email field injection**: `test' WHERE '1'='1@example.com`
- **Name field injection**: Nested UPDATE statements

### Search/Pagination Tests
- **Search parameter injection**: UNION-based attacks
- **Pagination parameter injection**: Malicious page/perPage values

### Comprehensive Attack Vectors
- Comment-based: `--, #, /* */`
- Boolean-based: `OR 1=1, AND 1=1`
- UNION-based: `UNION SELECT`
- Stacked queries: `; DROP TABLE`
- URL-encoded attacks
- Special characters handling

## What These Tests Verify

1. **No SQL Execution**: Malicious SQL code is never executed
2. **Data Integrity**: All input is treated as literal strings
3. **Database Safety**: Tables remain intact after attacks
4. **Error Handling**: No server errors (500) from injection attempts
5. **SeaORM Protection**: Parameterized queries prevent injection

## Running Tests

```bash
# Run all SQL injection tests
cargo test --features sqlite security::sql_injection

# Run with verbose output
cargo test --features sqlite security::sql_injection -- --nocapture

# Run specific test
cargo test --features sqlite test_sql_injection_in_email_during_user_creation

# Run without stopping on first failure
cargo test --features sqlite security::sql_injection --no-fail-fast
```

## Test Results

All 13 tests verify that:
- SeaORM's parameterized queries effectively prevent SQL injection
- Malicious input is safely stored as literal text
- No database corruption occurs
- Authentication cannot be bypassed
- Search and pagination parameters are safe

## Security Guarantees

These tests confirm that the lighter-auth service:
1. Uses parameterized queries for all database operations
2. Safely handles special characters (quotes, semicolons, etc.)
3. Cannot be compromised through common SQL injection techniques
4. Maintains data integrity even with malicious input
5. Returns appropriate error codes (400/404) instead of exposing errors

## Test Architecture

- **Framework**: actix-web test utilities with `service!` macro
- **Database**: In-memory SQLite (isolated per test)
- **Setup**: Uses `testing::setup` utilities for test users
- **Assertions**: Verify both operation results and database state

## Implementation Details

Location: `/Users/gerianoadikaputra/Programs/Own/lighter/auth/tests/security/sql_injection_test.rs`

- **Lines of code**: 802
- **Test count**: 13
- **Coverage**: All major CRUD operations
- **Attack patterns**: 10+ different SQL injection techniques

## Related Documentation

- SeaORM Security: https://www.sea-ql.org/SeaORM/docs/security/
- OWASP SQL Injection: https://owasp.org/www-community/attacks/SQL_Injection
- Project CLAUDE.md: Guidelines for secure development
