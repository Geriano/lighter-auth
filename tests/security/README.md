# Security Tests

Comprehensive security tests verifying that the lighter-auth service is protected against common web vulnerabilities including SQL injection and XSS (Cross-Site Scripting) attacks.

## Test Suites

### 1. SQL Injection Prevention Tests
Location: `sql_injection_test.rs`
Tests: 13
Coverage: SeaORM's parameterized queries

### 2. XSS Prevention Tests
Location: `xss_test.rs`
Tests: 12
Coverage: JSON encoding and Content-Type security

---

## SQL Injection Test Coverage

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

### SQL Attack Vectors
- Comment-based: `--, #, /* */`
- Boolean-based: `OR 1=1, AND 1=1`
- UNION-based: `UNION SELECT`
- Stacked queries: `; DROP TABLE`
- URL-encoded attacks
- Special characters handling

---

## XSS Test Coverage

### Stored XSS Tests
- **Name field XSS**: `<script>alert('XSS')</script>`
- **Email field XSS**: `test+<script>alert(1)</script>@example.com`
- **Username field XSS**: `admin<img src=x onerror=alert(1)>`
- **Event handler XSS**: `"><img src=x onerror=alert(1)>`
- **SVG-based XSS**: `<svg onload=alert(1)>`

### Reflected XSS Tests
- **Search parameter XSS**: Script injection in search queries
- **Filter parameter XSS**: XSS in URL parameters

### XSS Attack Vectors
- Classic script tags: `<script>alert('XSS')</script>`
- IMG with onerror: `<img src=x onerror=alert(1)>`
- SVG with onload: `<svg onload=alert(1)>`
- JavaScript protocol: `javascript:alert(1)`
- IFrame injection: `<iframe src="javascript:alert(1)">`
- Body onload: `<body onload=alert(1)>`
- Quote breaking: `"><script>alert(1)</script>`
- Event handlers: onclick, onerror, onload
- URL-encoded XSS: `%3Cscript%3E...`
- Unicode-encoded XSS: `\u003cscript\u003e...`
- Case variations: `<ScRiPt>alert('XSS')</ScRiPt>`
- Data URIs: `<img src="data:text/html,<script>...">`

### Update/Retrieve Tests
- **User update with XSS**: Verify safe handling during updates
- **Store and retrieve lifecycle**: Full lifecycle XSS safety
- **JSON encoding verification**: Dangerous characters properly encoded

---

## What These Tests Verify

### SQL Injection Protection
1. **No SQL Execution**: Malicious SQL code is never executed
2. **Data Integrity**: All input is treated as literal strings
3. **Database Safety**: Tables remain intact after attacks
4. **Error Handling**: No server errors (500) from injection attempts
5. **SeaORM Protection**: Parameterized queries prevent injection

### XSS Protection
1. **JSON Encoding**: Responses are properly JSON-encoded
2. **Content-Type Safety**: Responses served as `application/json`
3. **No Script Execution**: Script tags stored as literal strings
4. **Special Character Handling**: `< > " ' &` safely handled
5. **Lifecycle Safety**: XSS payloads safe during store and retrieve
6. **Valid JSON**: All responses parse as valid JSON

---

## Running Tests

```bash
# Run all security tests (SQL injection + XSS)
cargo test --features sqlite security -- --test-threads=1

# Run only SQL injection tests
cargo test --features sqlite security::sql_injection

# Run only XSS tests
cargo test --features sqlite security::xss

# Run with verbose output
cargo test --features sqlite security -- --nocapture

# Run specific test
cargo test --features sqlite test_xss_in_name_field_classic_script_tag

# Run without stopping on first failure
cargo test --features sqlite security --no-fail-fast
```

## Test Results

**Total Tests**: 25 (13 SQL injection + 12 XSS)
**Success Rate**: 100%
**Execution Time**: ~130-160 seconds (sequential execution)

### SQL Injection Tests (13 tests)
- SeaORM's parameterized queries effectively prevent SQL injection
- Malicious input is safely stored as literal text
- No database corruption occurs
- Authentication cannot be bypassed
- Search and pagination parameters are safe

### XSS Tests (12 tests)
- JSON encoding prevents script execution
- Responses are valid JSON and cannot break out of string context
- Content-Type header set to `application/json`
- All XSS vectors safely neutralized
- Full lifecycle (store → retrieve) verified safe

## Security Guarantees

These comprehensive tests confirm that the lighter-auth service:

### Against SQL Injection
1. Uses parameterized queries for all database operations
2. Safely handles special characters (quotes, semicolons, etc.)
3. Cannot be compromised through common SQL injection techniques
4. Maintains data integrity even with malicious input
5. Returns appropriate error codes (400/404) instead of exposing errors

### Against XSS
1. All API responses are JSON-encoded
2. Dangerous characters (`< > " ' &`) handled safely within JSON strings
3. Content-Type header prevents browser HTML interpretation
4. Script tags and event handlers stored as literal strings
5. No executable JavaScript in API responses
6. URL-encoded and Unicode-encoded XSS attempts neutralized

## Test Architecture

- **Framework**: actix-web test utilities with `service!` macro
- **Database**: In-memory SQLite (isolated per test)
- **Setup**: Uses `testing::setup` utilities for test users and authentication
- **Assertions**: Verify operation results, response format, and database state
- **Coverage**: All major CRUD operations and edge cases

## Implementation Details

### SQL Injection Tests
- **Location**: `tests/security/sql_injection_test.rs`
- **Lines of code**: ~802
- **Test count**: 13
- **Attack patterns**: 10+ different SQL injection techniques

### XSS Tests
- **Location**: `tests/security/xss_test.rs`
- **Lines of code**: ~850
- **Test count**: 12
- **Attack vectors**: 13+ different XSS techniques

## Key Insights

### How JSON Encoding Prevents XSS

The service is protected against XSS not through explicit sanitization, but through **JSON encoding**:

1. **String Context**: All user data is within JSON string values
2. **No HTML Rendering**: API responses are `application/json`, not HTML
3. **Safe Deserialization**: JSON parsers handle escaping automatically
4. **Cannot Break Context**: Even `<script>` tags remain within string boundaries

Example:
```json
{
  "name": "<script>alert('XSS')</script>"
}
```

When a client deserializes this JSON, they get the literal string `<script>alert('XSS')</script>`, which cannot execute because:
- It's inside a JSON string value
- The Content-Type is `application/json`
- Browsers won't interpret JSON responses as HTML

### How SeaORM Prevents SQL Injection

SeaORM uses **parameterized queries** which separate SQL logic from data:

```rust
// Safe: Parameter is bound separately
User::find()
    .filter(user::Column::Email.eq(user_input))
    .one(db)
    .await?

// This becomes: SELECT * FROM users WHERE email = ?
// With parameter: user_input
```

Even if `user_input` contains `' OR '1'='1`, it's treated as a literal string, not SQL code.

## Related Documentation

- **SeaORM Security**: https://www.sea-ql.org/SeaORM/docs/security/
- **OWASP SQL Injection**: https://owasp.org/www-community/attacks/SQL_Injection
- **OWASP XSS Prevention**: https://cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet.html
- **Project CLAUDE.md**: Guidelines for secure development

## Continuous Security

These tests should be run:
- ✅ Before every commit (pre-commit hook)
- ✅ In CI/CD pipeline (automated)
- ✅ Before production deployment
- ✅ After dependency updates
- ✅ When adding new endpoints

**Never skip security tests!**
