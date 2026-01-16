# Security Features - Blackjack API

**Last Updated:** January 15, 2026  
**Milestone:** 8 (In Progress)  
**Security Level:** Production-Ready (Password & Authentication)

## Overview

The Blackjack Multi-Player API implements comprehensive security measures to protect user accounts, game data, and API access. This document outlines the security features, threat mitigations, and best practices.

---

## 🔐 Password Security (Milestone 8)

### Hashing Algorithm

**Argon2id** - Winner of the Password Hashing Competition (PHC) and OWASP recommended algorithm.

#### Why Argon2id?
- **Hybrid approach**: Combines Argon2i (timing attack resistant) and Argon2d (GPU attack resistant)
- **Memory-hard**: Requires significant RAM, making brute-force attacks expensive
- **Configurable**: Tunable parameters for future-proofing
- **Industry standard**: Recommended by OWASP, NIST, and security experts

#### Implementation Parameters

```rust
Memory cost:  19456 KiB (19 MiB)  // OWASP recommended
Time cost:    2 iterations         // OWASP recommended
Parallelism:  1 thread             // Single-threaded for simplicity
Salt:         16 bytes (random)    // Generated per password
Output:       PHC string format    // Standard format with all params
```

**Example hash:**
```
$argon2id$v=19$m=19456,t=2,p=1$<base64_salt>$<base64_hash>
```

### Password Complexity Requirements

All user passwords must meet these requirements:

| Requirement | Description | Example |
|------------|-------------|---------|
| **Length** | Minimum 8 characters | `MyP@ssw0rd` (10 chars) |
| **Uppercase** | At least 1 uppercase letter (A-Z) | `M`, `P` |
| **Lowercase** | At least 1 lowercase letter (a-z) | `y`, `s`, `w`, `r`, `d` |
| **Digit** | At least 1 number (0-9) | `0` |
| **Special** | At least 1 special character | `@` |

**Valid special characters:**
```
! @ # $ % ^ & * ( ) - _ = + [ ] { } | \ ; : ' " , . < > ? /
```

**Valid password examples:**
- `MyP@ssw0rd` ✅
- `Secure#Pass123` ✅
- `Test!User2024` ✅

**Invalid password examples:**
- `password` ❌ (no uppercase, no digit, no special)
- `Password` ❌ (no digit, no special)
- `Pass123` ❌ (too short, no special)
- `P@ss` ❌ (too short, no digit)

### Password Operations

#### Registration
1. **Validation**: Email format + password complexity checked
2. **Hashing**: Password hashed with Argon2id (never stored as plaintext)
3. **Storage**: Only hash stored in database
4. **Response**: Returns user ID, never echoes password

#### Login
1. **Lookup**: User retrieved by email
2. **Active check**: Account `is_active` status verified
3. **Verification**: Constant-time password comparison
4. **Timestamp**: `last_login` updated on success
5. **Logging**: Failed attempts logged (email only, not password)

#### Password Change
1. **Authentication**: Old password verified
2. **Validation**: New password complexity checked
3. **Hashing**: New password hashed with fresh salt
4. **Update**: Hash replaced, old hash discarded
5. **Token invalidation**: All JWT tokens should be invalidated (API layer)

### Security Properties

✅ **Constant-Time Verification**: Prevents timing attacks  
✅ **Random Salt**: Unique per password, prevents rainbow tables  
✅ **Memory-Hard**: Expensive to parallelize on GPUs  
✅ **No Plaintext Storage**: Passwords never stored unencrypted  
✅ **No Plaintext Logging**: Passwords never appear in logs  
✅ **No Plaintext Response**: Passwords never echoed in API responses  

---

## 👥 Access Control (Milestone 8)

### Role-Based Access Control (RBAC)

The API implements a **Role-Based Access Control** system for game management.

#### Game Roles

| Role | Description | Assigned To |
|------|-------------|-------------|
| **Creator** | Full control over the game | User who created the game |
| **Player** | Can perform own actions only | Users who enrolled in the game |
| **Spectator** | Read-only access (future) | Invited observers (not implemented) |

#### Game Permissions

| Permission | Description | Allowed Roles |
|-----------|-------------|---------------|
| `InvitePlayers` | Invite other users to join | Creator only |
| `KickPlayers` | Remove players from game | Creator only |
| `CloseEnrollment` | Manually close enrollment period | Creator only |
| `FinishGame` | Manually finish the game | Creator only |
| `ModifySettings` | Change game configuration | Creator only |

**Note:** Players can always perform their own gameplay actions (draw, stand, set ace value) regardless of role.

#### Permission Checking

```rust
// Check if user can perform an action
if !game.can_user_perform(user_id, GamePermission::KickPlayers) {
    return Err(GameError::InsufficientPermissions);
}

// Check if user is the creator
if !game.is_creator(user_id) {
    return Err(GameError::NotGameCreator);
}

// Get user's role
match game.get_participant_role(user_id) {
    Some(GameRole::Creator) => { /* allow */ },
    Some(GameRole::Player) => { /* limited */ },
    None => return Err(GameError::NotAParticipant),
}
```

### User Account Management

#### Account Status

Each user has an `is_active` boolean field:
- **`true`** (default): Account is active, user can login
- **`false`**: Account is deactivated, login returns `403 AccountInactive`

**Use cases for deactivation:**
- User requested account suspension
- Admin action (abuse, policy violation)
- Temporary lockout (future: after failed login attempts)

#### Account Operations

```rust
// Deactivate account
user.deactivate();
user.is_active = false;

// Activate account
user.activate();
user.is_active = true;

// Check status
if !user.is_account_active() {
    return Err(GameError::AccountInactive);
}
```

---

## 🔑 Authentication (JWT)

### Token Structure

**Claims:**
```json
{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "player@example.com",
  "exp": 1737849600
}
```

**Security Features:**
- HS256 signature algorithm
- Configurable expiration (default: 24 hours)
- Secret key from environment variable
- Token validated on every protected endpoint

### Protected Endpoints

All endpoints except health checks and registration require valid JWT:

```
Authorization: Bearer <token>
```

**Response on invalid/missing token:**
```json
{
  "error": "Unauthorized",
  "message": "Invalid or missing authentication token",
  "code": "UNAUTHORIZED"
}
```

---

## 🛡️ Threat Mitigation

### Threats Addressed (Milestone 8)

| Threat | Mitigation | Status |
|--------|-----------|--------|
| **Rainbow Table Attack** | Random salt per password | ✅ Mitigated |
| **Brute Force Password** | Memory-hard Argon2id | ✅ Mitigated |
| **Timing Attack** | Constant-time comparison | ✅ Mitigated |
| **Password Reuse** | Password complexity requirements | ✅ Mitigated |
| **Weak Passwords** | Validation with 4 requirements | ✅ Mitigated |
| **Privilege Escalation** | RBAC with permission checks | ✅ Mitigated |
| **Unauthorized Game Control** | Creator-only permissions | ✅ Mitigated |
| **Account Enumeration** | Generic "Invalid credentials" message | ✅ Mitigated |

### Threats Partially Addressed

| Threat | Current Mitigation | Future Enhancement |
|--------|-------------------|-------------------|
| **Account Lockout** | Account deactivation exists | Auto-lockout after N failed attempts |
| **Session Hijacking** | JWT with expiration | Token rotation, refresh tokens |
| **CSRF** | Stateless JWT | CSRF tokens for state-changing ops |
| **XSS** | Server-side only | Security headers (M8 remaining) |

### Threats Not Yet Addressed

| Threat | Status | Planned Mitigation |
|--------|--------|-------------------|
| **DDoS** | Rate limiting exists | Enhanced per-IP rate limiting |
| **SQL Injection** | Not applicable (in-memory) | Parameterized queries when DB added |
| **Audit Trail** | Partial logging | Comprehensive audit log table |
| **MFA** | Not implemented | TOTP-based 2FA (future) |

---

## 📊 Security Logging

### What Gets Logged

**Registration:**
- ✅ Successful registrations (email, user_id)
- ✅ Failed registrations (duplicate email)
- ❌ Passwords (never logged)

**Login:**
- ✅ Successful logins (email, user_id)
- ✅ Failed logins (email only, reason)
- ✅ Inactive account login attempts
- ❌ Passwords (never logged)

**Password Change:**
- ✅ Successful password changes (user_id)
- ✅ Failed attempts (incorrect old password)

**Access Control:**
- ⏳ Permission denials (user_id, action, resource) - planned

### Log Levels

```rust
tracing::info!()   // Successful operations
tracing::warn!()   // Failed attempts, security events
tracing::error!()  // System errors, critical failures
tracing::debug!()  // Detailed flow for debugging
```

---

## 🔧 Configuration

### Security Settings

**Current (config.toml):**
```toml
[jwt]
secret = "dev-secret-key-change-in-production"
expiration_hours = 24
```

**Planned (Milestone 8):**
```toml
[security]
password_min_length = 8
password_require_uppercase = true
password_require_lowercase = true
password_require_number = true
password_require_special = true
max_login_attempts = 5
lockout_duration_minutes = 15

[security.argon2]
memory_cost = 19456  # KiB
time_cost = 2
parallelism = 1
```

### Environment Variables

**Current:**
- `BLACKJACK_JWT_SECRET` - Override JWT secret (required in production)
- `RUST_LOG` - Logging level

**Planned:**
- `BLACKJACK_SECURITY_PASSWORD_MIN_LENGTH`
- `BLACKJACK_SECURITY_MAX_LOGIN_ATTEMPTS`
- `BLACKJACK_SECURITY_LOCKOUT_DURATION_MINUTES`

---

## ✅ Security Checklist

### Implemented (Milestone 8 - Current)
- [x] Argon2id password hashing
- [x] Password complexity validation
- [x] Email format validation
- [x] Constant-time password verification
- [x] Random salt generation per password
- [x] Account status tracking (`is_active`)
- [x] Last login timestamp
- [x] Role-based access control (RBAC)
- [x] Game permission system
- [x] Secure password change endpoint
- [x] Generic error messages (no account enumeration)
- [x] Security logging (auth events)

### In Progress (Milestone 8 - Remaining)
- [ ] Security headers middleware (X-Content-Type-Options, etc.)
- [ ] Audit logging table
- [ ] Failed login attempt tracking
- [ ] Account lockout after N failures
- [ ] Enhanced rate limiting per-IP

### Future Enhancements
- [ ] Token refresh mechanism
- [ ] Password reset via email
- [ ] Multi-factor authentication (MFA/TOTP)
- [ ] Session management
- [ ] IP-based geolocation blocking
- [ ] Anomaly detection
- [ ] Security event notifications

---

## 📖 Best Practices for Developers

### Password Handling
1. **Never** log passwords (not even hashed)
2. **Never** echo passwords in responses
3. **Always** validate before hashing
4. **Always** use constant-time comparison
5. **Never** reuse salts

### Access Control
1. **Always** check permissions before sensitive operations
2. **Fail closed** - deny by default
3. **Validate** user is participant before role check
4. **Log** permission denials for security monitoring

### Error Messages
1. **Generic** authentication errors (don't reveal if email exists)
2. **Specific** validation errors (help users fix input)
3. **Detailed** logs (for debugging), **generic** responses (for security)

### Configuration
1. **Never** commit secrets to version control
2. **Use** environment variables for production secrets
3. **Rotate** JWT secrets periodically
4. **Document** security-sensitive configuration

---

## 🔗 Related Documentation

- [README.md](../README.md) - Project overview
- [PRD.md](PRD.md) - Full product requirements (Milestone 8)
- [API Documentation](../README.md#api-endpoints) - Endpoint reference
- [QUICK_REFERENCE.md](QUICK_REFERENCE.md) - Quick API reference

---

**Questions or Security Concerns?**  
Please report security issues privately to the project maintainers.

**Last Review:** January 15, 2026  
**Next Review:** Milestone 8 completion
