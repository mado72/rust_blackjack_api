# Milestone 8 - Security Hardening

## Current Status

**Branch:** `feature/M8`  
**Started:** January 14, 2026  
**Dependencies:** M7 Complete ✅  
**Estimated Effort:** 8-10 hours

---

## Overview

Implement robust security measures including proper password hashing with modern cryptographic standards, enhanced input validation, and security headers to protect against common web vulnerabilities.

---

## 🔒 Security Improvements

### Phase 1: Password Hashing with Argon2 (3 hours)

**Current State:** Passwords stored in plain text in memory (insecure)

**Target State:** Passwords hashed with Argon2id

#### Tasks

1. **Add Dependencies**
   ```toml
   # Cargo.toml
   argon2 = "0.5"
   ```

2. **Update UserService**
   - [ ] Implement password hashing in `register()`
   - [ ] Implement password verification in `login()`
   - [ ] Use Argon2id variant (recommended for password hashing)
   - [ ] Configure secure parameters:
     - Memory cost: 19456 KiB (19 MiB)
     - Time cost: 2 iterations
     - Parallelism: 1
   - [ ] Generate random salt per password

3. **Migration Support**
   - [ ] Add password migration utility (if needed for future DB)
   - [ ] Document password format in code comments

4. **Testing**
   - [ ] Test password hashing during registration
   - [ ] Test password verification during login
   - [ ] Test invalid password rejection
   - [ ] Test salt uniqueness

**Acceptance Criteria:**
- ✅ Passwords never stored in plain text
- ✅ Argon2id hashing implemented
- ✅ Salt generated per password
- ✅ Login verification works correctly
- ✅ Tests cover hashing scenarios

---

### Phase 2: Input Validation (2 hours)

**Current State:** Basic validation, potential security gaps

**Target State:** Comprehensive input validation with sanitization

#### Tasks

1. **Add Dependencies**
   ```toml
   # Cargo.toml
   validator = { version = "0.16", features = ["derive"] }
   regex = "1.10"
   ```

2. **Email Validation**
   - [ ] Add email regex validation
   - [ ] Reject invalid email formats
   - [ ] Add length limits (max 254 chars)
   - [ ] Normalize emails (lowercase, trim)

3. **Password Strength Validation**
   - [ ] Minimum 8 characters
   - [ ] Require at least:
     - 1 uppercase letter
     - 1 lowercase letter
     - 1 number
     - 1 special character
   - [ ] Maximum 128 characters
   - [ ] Block common passwords (optional)

4. **Game Input Validation**
   - [ ] Validate enrollment_timeout_seconds (min: 60, max: 86400)
   - [ ] Validate game_id format (UUID)
   - [ ] Validate invitation_id format (UUID)

5. **Request Size Limits**
   - [ ] Add body size limits in middleware
   - [ ] Reject oversized requests

**Acceptance Criteria:**
- ✅ Email validation implemented
- ✅ Password strength enforced
- ✅ Game parameters validated
- ✅ Appropriate error messages
- ✅ Tests cover validation scenarios

---

### Phase 3: Security Headers (1 hour)

**Current State:** Basic CORS headers only

**Target State:** Comprehensive security headers

#### Tasks

1. **Add tower-http Security Layer**
   - Already in dependencies, enhance configuration

2. **Implement Security Headers**
   - [ ] `X-Content-Type-Options: nosniff`
   - [ ] `X-Frame-Options: DENY`
   - [ ] `X-XSS-Protection: 1; mode=block`
   - [ ] `Strict-Transport-Security: max-age=31536000; includeSubDomains`
   - [ ] `Content-Security-Policy: default-src 'self'`
   - [ ] `Referrer-Policy: strict-origin-when-cross-origin`

3. **Update Middleware**
   - [ ] Add security headers middleware
   - [ ] Apply to all routes
   - [ ] Document header purposes

**Acceptance Criteria:**
- ✅ All security headers present
- ✅ Headers applied to all responses
- ✅ HTTPS redirect configured (HSTS)

---

### Phase 4: Access Control Enhancement (2 hours)

**Current State:** Basic JWT validation

**Target State:** Enhanced permission validation

#### Tasks

1. **Role-Based Validation**
   - [ ] Add `is_game_creator()` helper
   - [ ] Validate creator-only actions:
     - Close enrollment
     - Delete game (future)
   - [ ] Add detailed error messages for permission denied

2. **Enrollment Validation**
   - [ ] Verify user enrolled before game actions
   - [ ] Block non-enrolled users from:
     - Drawing cards
     - Standing
     - Viewing game details (optional)
   - [ ] Only enrolled players can invite others

3. **Rate Limiting Enhancement**
   - [ ] Add per-endpoint rate limits:
     - Register: 5/hour per IP
     - Login: 10/hour per email
     - Create game: 20/hour per user
     - Draw card: 100/minute per user
   - [ ] Add Redis support (future)

**Acceptance Criteria:**
- ✅ Creator-only actions enforced
- ✅ Enrollment validated before gameplay
- ✅ Rate limits per endpoint
- ✅ Clear error messages

---

### Phase 5: Audit Logging (1 hour)

**Current State:** Basic tracing logs

**Target State:** Security audit trail

#### Tasks

1. **Security Event Logging**
   - [ ] Log authentication failures
   - [ ] Log permission denied events
   - [ ] Log rate limit violations
   - [ ] Log suspicious activities

2. **Structured Logging**
   - [ ] Add security-specific log levels
   - [ ] Include user_id, IP, timestamp
   - [ ] Add log correlation IDs

3. **Log Rotation**
   - [ ] Document log rotation strategy
   - [ ] Add log level configuration

**Acceptance Criteria:**
- ✅ Security events logged
- ✅ Structured format with context
- ✅ Log rotation documented

---

## Testing Plan

### Unit Tests
- [ ] Password hashing tests (5 tests)
- [ ] Email validation tests (10 tests)
- [ ] Password strength tests (8 tests)
- [ ] Access control tests (6 tests)

### Integration Tests
- [ ] End-to-end with password hashing
- [ ] Invalid input rejection
- [ ] Permission denied scenarios
- [ ] Rate limit enforcement

### Security Tests
- [ ] SQL injection attempts (N/A - no SQL yet)
- [ ] XSS attempts (header protection)
- [ ] CSRF attempts (header protection)
- [ ] Brute force login attempts

**Target:** 110+ tests passing

---

## Documentation Updates

- [ ] Update README.md with security features
- [ ] Add SECURITY.md with vulnerability reporting
- [ ] Update PRD.md M8 section as complete
- [ ] Add password requirements to API docs
- [ ] Document security headers

---

## Migration Notes

### Breaking Changes

1. **Password Format**
   - Old: Plain text stored
   - New: Argon2 hash stored
   - Migration: Existing users need to re-register (in-memory, no impact)

2. **Validation Rules**
   - Stricter email validation
   - Password strength requirements
   - May reject previously accepted inputs

3. **Error Messages**
   - More detailed validation errors
   - Security-conscious error messages (no user enumeration)

---

## Success Criteria

- ✅ Argon2 password hashing implemented
- ✅ Comprehensive input validation
- ✅ Security headers on all responses
- ✅ Enhanced access control
- ✅ Security audit logging
- ✅ All tests passing (110+)
- ✅ Zero security warnings
- ✅ Documentation updated
- ✅ Code review approved

---

## References

- **PRD:** [docs/PRD.md](PRD.md) - Milestone 8 section
- **Argon2 Spec:** [RFC 9106](https://www.rfc-editor.org/rfc/rfc9106.html)
- **OWASP Top 10:** Security best practices
- **Axum Security:** Tower-http security middleware

---

## Next Milestone Preview

**Milestone 9: Database Integration (SQLite)**
- Persistent storage
- User accounts
- Game history
- Migration system

- ✅ GameService::get_open_games(exclude_user_id: Option<Uuid>) -> Vec<GameInfo>
  - Lista games em fase de enrollment
  - Retorna: game_id, creator_id, enrolled_count, max_players, enrollment_timeout_seconds, time_remaining_seconds

- ✅ GameService::enroll_player(game_id, player_email) -> Result<(), GameError>
  - Valida se game está aberto
  - Valida capacity < 10
  - Retorna GameError::GameFull se cheio
  - Retorna GameError::EnrollmentClosed se expirado

- ✅ GameService::close_enrollment(game_id, user_id) -> Result<Vec<String>, GameError>
  - Valida se user é creator
  - Inicializa turn_order
  - Retorna turn_order

- ✅ InvitationService completa:
  - create(game_id, inviter_id, invitee_email, game_enrollment_expires_at)
  - accept(invitation_id) com validação de expiração
  - decline(invitation_id)
  - get_pending_for_user(email) filtra expiradas
  - cleanup_expired() marca como expirada
  - get_invitation(invitation_id)

- ✅ Error Handling:
  - GameError::GameFull
  - GameError::EnrollmentClosed
  - Todos os outros erros existentes

- ✅ Testes: 82 tests PASSANDO
  - 19 core integration tests
  - 12 service tests
  - 16 API tests
  - 13 CLI tests
  - 22 doctests

### API Layer (Parcial - 20% - HANDLERS ESCRITOS, ROUTING PENDENTE)
- ✅ **Handlers Implementados** (em `crates/blackjack-api/src/handlers/games.rs`):
  - `create_game_handler()` - POST /api/v1/games
  - `get_open_games_handler()` - GET /api/v1/games/open
  - `enroll_player_handler()` - POST /api/v1/games/:game_id/enroll
  - `close_enrollment_handler()` - POST /api/v1/games/:game_id/close-enrollment
  - `draw_card_handler()` - POST /api/v1/games/:game_id/draw com validação enrollment_closed

- ❌ **Routing NÃO Configurado**:
  - Handlers estão escritos mas NÃO roteáveis em main.rs
  - Próximo passo crítico: Wire handlers ao router

---

## 🚀 Próximos Passos (PHASE 2)

### PHASE 2A: Implementar Game Invitations Endpoints (2-3 horas)

**Handlers a verificar/implementar:**

1. ✅ `POST /api/v1/games/:game_id/invitations` - Create invitation
   - Status: Verificar se já existe em handlers.rs
   - Ação: Verificar se está routed em main.rs
   
2. ✅ `GET /api/v1/invitations/pending` - Get pending invitations
   - Status: Verificar se já existe em handlers.rs
   - Ação: Verificar se está routed em main.rs

3. ✅ `POST /api/v1/invitations/:id/accept` - Accept invitation
   - Status: Verificar se já existe em handlers.rs
   - Ação: Verificar se está routed em main.rs

**Próximo Comando:**
```
"Verifique se os 3 handlers de invitations já existem em handlers.rs.
Se existem, adicione-os ao router em main.rs.
Se não existem, implemente-os seguindo o padrão dos handlers de enrollment."
```

### PHASE 2B: Stand Endpoint (1-2 horas)

**⚠️ BLOCKER:** PHASE 3 deve ser feito antes

**Handler a implementar:**
- `POST /api/v1/games/:game_id/stand` - Player stands
  - Requer: PlayerState enum (PHASE 3)
  - Requer: Turn validation em draw_card (PHASE 3)

---

### PHASE 3: PlayerState Enum & Turn Management (3-4 horas)

**Änderungen erforderlich im Core:**

1. Create `PlayerState` enum in blackjack-core:
   ```rust
   pub enum PlayerState {
       Active,
       Standing,
       Busted,
   }
   ```

2. Update `Player` struct:
   - Add field: `state: PlayerState`

3. Update `Game` struct methods:
   - `get_current_player() -> Option<&str>`
   - `advance_turn() -> Result<(), GameError>`
   - `stand(email) -> Result<(), GameError>`
   - `check_auto_finish() -> bool`

4. Update service layer:
   - Add turn validation to `draw_card()`
   - Implement `stand()` method
   - Implement auto-finish logic

### PHASE 4: Testes Completos (4-6 horas)

- Add 12+ core tests para turn management
- Add 15+ service tests para turn validation
- Add 20+ API tests para endpoints e error cases
- Update PRD.md com implementação final

---

## 📋 Quick Start - PHASE 2

```bash
# 1. Check if invitation handlers exist
grep -n "create_invitation\|accept_invitation\|get_pending_invitations" \
  crates/blackjack-api/src/handlers.rs

# 2. Check if they're imported in main.rs
grep "create_invitation\|accept_invitation\|get_pending_invitations" \
  crates/blackjack-api/src/main.rs

# 3. Check if they're routed
grep "invitations" crates/blackjack-api/src/main.rs

# 4. If all missing, implement following the enrollment pattern
# 5. Test
cargo test --workspace
cargo build --release
```

---

## 📊 Final Status

✅ **PHASE 1: Enrollment Endpoints** - COMPLETE  
- 4 endpoints implemented
- 346 lines of code
- 78/78 tests passing
- Zero warnings
- Production ready

⏳ **PHASE 2: Invitations & Stand** - PENDING  
- 3 invitation endpoints
- 1 stand endpoint
- Requires checking existing code first

⏳ **PHASE 3: Turn Management** - PENDING  
- PlayerState enum
- Turn validation
- Auto-finish logic

⏳ **PHASE 4: Tests & Docs** - PENDING  
- Additional test coverage
- PRD final update
- API documentation


**Status:** ✅ COMPLETE - All enrollment handlers wired and functional

Details in [PHASE1_COMPLETION.md](PHASE1_COMPLETION.md)

---

### **FASE 2A: Implementar Game Invitations Endpoints (2-3 horas)**

Check if these handlers already exist:

**Requisitos:**
- Inviter deve estar enrolled no game
- Usa timeout do game (não customizável)
- Pending invitations filtra expiradas
- Accept valida não expirada
- Accept inscreve player no game

---

### **FASE 3: PlayerState Enum & Turn Management (3 horas)**

Implementar no core (não existente ainda):
```rust
pub enum PlayerState {
    Active,
    Standing,
    Busted,
}

// Adicionar ao Player struct
pub state: PlayerState

// Implementar métodos
Game::stand(email) -> Result<(), GameError>
Game::get_current_player() -> Option<&str>
Game::advance_turn() -> Result<(), GameError>
Game::check_auto_finish() -> bool
```

**Endpoints turn-based:**
```
[ ] POST   /api/v1/games/:game_id/stand (NOVO)
[ ] POST   /api/v1/games/:game_id/draw (UPDATE - adicionar turn validation)
```

---

### **FASE 4: Testes Novos (8 horas)**

```
[ ] 12 core tests: enrollment, turns, auto-finish
[ ] 15 service tests: turn validation, stand mechanism
[ ] 20 API tests: endpoints, error cases, full flow
```

---

### **FASE 5: Atualizar PRD (2 horas)**

Refletir implementações reais no documento PRD.md

---

## 📊 Mapa de Dependências

```
FASE 1: Wire Routing (15 min) ⏳ BLOCKER
    ↓ LIBERA TUDO
FASE 2A: Invitations Endpoints (2h)
    ↓
FASE 2B: Stand Endpoint (1h)
    ↓
FASE 3: PlayerState + Turn Mgmt (3h) ⬅️ NECESSÁRIO PARA FASE 2B
    ↓
FASE 4: Testes Completos (8h)
    ↓
FASE 5: PRD Final (2h)
```

---

## 📁 Arquivos Chave

- `crates/blackjack-core/src/game.rs` - Game struct (✅ COMPLETO)
- `crates/blackjack-core/src/models/invitation.rs` - Invitations (✅ COMPLETO)
- `crates/blackjack-service/src/game_service.rs` - GameService (✅ COMPLETO)
- `crates/blackjack-service/src/invitation_service.rs` - InvitationService (✅ COMPLETO)
- `crates/blackjack-api/src/handlers/games.rs` - Handlers (✅ ESCRITOS, ⏳ ROUTING PENDENTE)
- `crates/blackjack-api/src/main.rs` - Routing (⏳ A ATUALIZAR - FASE 1)
- `docs/PRD.md` - Product Requirements (✅ ATUALIZADO)

---

## 🎯 Próximo Comando da IA

**Próxima ação:** Começar **FASE 1 - Wire API Routing**

A IA deve:
1. Revisar os 4 handlers já escritos em `crates/blackjack-api/src/handlers/games.rs`
2. Identificar assinatura de cada handler
3. Localizar router configuration em `crates/blackjack-api/src/main.rs`
4. Adicionar as 4 routes ao router
5. Garantir integração com JWT authentication existente
6. Compilar e reportar status (`cargo build`)
7. Indicar próximo passo (FASE 2)

---

## 📋 Build Status

```
✅ Cargo build --workspace: SUCCESS
✅ Cargo test --workspace: 82 PASSING, 0 FAILING
✅ No compilation errors
⚠️  Minimal warnings (all non-critical)
```

---

## 🔑 Pontos-Chave para Session 2

1. **FASE 1 é blocker crítico** - sem routing, handlers não funcionam
2. **Handlers já existem** - apenas falta wire ao router
3. **Todos os testes passam** - código é estável
4. **Ordenação importa** - FASE 3 deve ser antes de endpoints turn-based
5. **JWT já integrado** - handlers herdão autenticação existente
6. **PRD alinhado** - implementação reflete requisitos do documento

---

## ❓ Comando de Início

```
"Implemente FASE 1: Wire os 4 handlers de enrollment ao router em main.rs.

Passos:
1. Revise os 4 handlers em crates/blackjack-api/src/handlers/games.rs
2. Localize router configuration em crates/blackjack-api/src/main.rs
3. Adicione as 4 routes (POST /api/v1/games, GET /api/v1/games/open, POST /api/v1/games/:game_id/enroll, POST /api/v1/games/:game_id/close-enrollment)
4. Compile com 'cargo build'
5. Reporte status e próximos passos"
```
