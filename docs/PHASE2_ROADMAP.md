# Next Steps - Milestone 7 PHASE 2

**Current Status:** PHASE 1 Complete ✅  
**Next Phase:** PHASE 2 - Invitations & Turn-Based Endpoints  
**Estimated Duration:** 6 hours total  

## Phase 2 Roadmap

### PHASE 2A: Game Invitation Endpoints (2 hours)

**Endpoints to implement:**

1. **POST /api/v1/games/:game_id/invitations** - Create invitation
   - Handler name: `create_invitation()` (already exists, verify it's wired)
   - Request: `{invitee_email: String}`
   - Response: `{invitation_id, message, invitee_email, expires_at}`
   - Validation: Inviter must be enrolled in game
   - Error: NotEnrolled (403), GameNotFound (404), InvitationExpired (410)

2. **GET /api/v1/invitations/pending** - Get pending invitations
   - Handler name: `get_pending_invitations()` (already exists, verify it's wired)
   - Response: `{invitations: [InvitationInfo], count}`
   - Filter: Only pending invitations for authenticated user
   - Auto-update: Mark expired invitations as expired

3. **POST /api/v1/invitations/:id/accept** - Accept invitation
   - Handler name: `accept_invitation()` (already exists, verify it's wired)
   - Response: `{game_id, message, player_count}`
   - Validation: Invitation not expired, invitee matches JWT
   - Action: Auto-enroll player in game
   - Error: InvitationExpired (410), NotInvitee (403)

**Status Check Required:**
- Are these handlers already in handlers.rs? → CHECK FIRST
- Are they already imported in main.rs? → CHECK FIRST
- Are they already routed? → CHECK FIRST

---

### PHASE 2B: Stand Endpoint (1 hour)

**⚠️ BLOCKERS:** PHASE 3 must be completed first

**Endpoint to implement:**

1. **POST /api/v1/games/:game_id/stand** - Player stands
   - Handler name: `stand()` (already exists, verify it's wired)
   - Response: `{points, busted, message, game_finished}`
   - Validation: Enrollment closed, is player's turn
   - Action: Mark player as standing, advance turn, check auto-finish
   - Error: NotYourTurn (409), EnrollmentOpen (403)

**Dependency:** PHASE 3 (PlayerState enum must exist)

---

## PHASE 3: PlayerState & Turn Management (3 hours)

**Core changes required (before PHASE 2B):**

1. **Create PlayerState Enum in blackjack-core**
   ```rust
   pub enum PlayerState {
       Active,      // Currently playing
       Standing,    // Chose to stop drawing
       Busted,      // Exceeded 21
   }
   ```

2. **Update Player Struct**
   - Add field: `state: PlayerState` (default: Active)

3. **Implement Turn Methods in Game Struct**
   - `get_current_player() -> Option<&str>` - Get email of current player
   - `advance_turn() -> Result<(), GameError>` - Move to next active player
   - `stand(email) -> Result<(), GameError>` - Mark player as standing
   - `check_auto_finish() -> bool` - Check if all players done

4. **Update draw_card in Game Struct**
   - Add validation: enrollment_closed must be true
   - Add validation: only current player can draw
   - After draw: Check auto-finish and advance turn if busted

5. **Update GameService Methods**
   - `draw_card()` - Add turn validation
   - `stand()` - New method to mark standing and advance

---

## PHASE 4: Tests & Documentation (8 hours)

After PHASE 2 & 3 complete:

1. **Add Core Tests**
   - Test PlayerState enum
   - Test stand() method
   - Test turn advancement
   - Test auto-finish detection
   - Test turn validation in draw_card

2. **Add Service Tests**
   - Test draw_card with turn validation
   - Test stand with turn advancement
   - Test auto-finish logic
   - Test error cases

3. **Add API Tests**
   - Test stand endpoint
   - Test turn validation errors
   - Test invitation acceptance
   - Test invitation expiration

4. **Update Documentation**
   - Update PRD.md with final implementation
   - Create API reference guide
   - Update curl examples
   - Update Postman collection

---

## Testing Strategy

### Quick Verification (15 min)
```bash
# After each phase, run:
cargo test --workspace
cargo build --release
cargo check -p blackjack-api
```

### Full Test Suite (5 min)
```bash
cargo test --all
```

### Manual Testing
Use Postman or curl to verify:
1. Create game → List games → Enroll players → Close enrollment
2. Create invitation → List pending → Accept invitation
3. Draw card (with turn validation) → Stand → Game finishes

---

## Implementation Tips

### Handlers Implementation
- Copy structure from existing handlers (handlers.rs has good examples)
- Use `#[tracing::instrument]` for logging
- Include comprehensive error handling
- Document with examples

### Error Handling
- Use existing `ApiError` enum
- Return proper HTTP status codes
- Provide meaningful error messages
- Log warnings/errors appropriately

### Testing
- Add tests to `/tests/` directory
- Test happy path and error cases
- Verify error messages are correct
- Check logging output

---

## Success Criteria

✅ All endpoints implemented and wired  
✅ All tests passing (expect ~100 tests total)  
✅ No clippy warnings  
✅ Compilation succeeds  
✅ Documentation complete  
✅ Postman collection updated  

---

## References

- **Existing Handlers:** See `crates/blackjack-api/src/handlers.rs`
- **Router Config:** See `crates/blackjack-api/src/main.rs`
- **Service Methods:** See `crates/blackjack-service/src/lib.rs`
- **Core Logic:** See `crates/blackjack-core/src/lib.rs`
- **Tests:** See `crates/*/tests/` directories

---

## Command Reference

```bash
# Build and test
cargo build --release
cargo test --workspace

# Check specific crate
cargo test -p blackjack-api
cargo test -p blackjack-service

# Run with logging
RUST_LOG=debug cargo run -p blackjack-api

# Format code
cargo fmt

# Check for issues
cargo clippy -- -D warnings
```

---

**Ready to proceed with PHASE 2A?**  
→ Check if invitation handlers already exist  
→ Verify they're imported in main.rs  
→ Verify they're routed in Router  
→ If all already done, only need to test them
