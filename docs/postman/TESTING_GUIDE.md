# Phase 2 Testing Guide

## Quick Start

### Option 1: Automated PowerShell Test (Recommended)

```powershell
# Terminal 1: Start server
cd C:\Users\mado7\Proj\rust_blackjack
$env:RUST_LOG="debug"
cargo run -p blackjack-api

# Terminal 2: Run automated tests
cd C:\Users\mado7\Proj\rust_blackjack
.\docs\postman\test_phase2.ps1
```

### Option 2: Manual Testing with HTTP File

1. Start server:
```powershell
cd C:\Users\mado7\Proj\rust_blackjack
$env:RUST_LOG="debug"
cargo run -p blackjack-api
```

2. Open `docs/postman/phase2_manual_tests.http` in VS Code

3. Install "REST Client" extension if needed

4. Click "Send Request" above each test

### Option 3: Postman Collection

Import `docs/postman/Blackjack_API.postman_collection.json` into Postman

---

## Test Scenarios

### Scenario 1: Complete Game Flow (Happy Path) ✅

1. **Register 3 users** (alice, bob, charlie)
2. **Login all users** → Get JWT tokens
3. **Alice creates game** → Get game_id
4. **List open games** → Verify game appears
5. **Bob enrolls** in game
6. **Charlie enrolls** in game
7. **Alice creates invitation** for diana@test.com
8. **Alice closes enrollment** → Turn order established
9. **Get game state** → Check current_turn_player
10. **Players take turns:**
    - Current player draws card
    - Turn advances automatically
    - Or player stands
11. **Game auto-finishes** when all players done
12. **Get results** → Winner, scores, etc.

**Expected:** All operations succeed, proper turn management, auto-finish works

---

### Scenario 2: Turn Validation ✅

1. Create game, enroll players, close enrollment
2. Check whose turn it is (e.g., Alice)
3. **Try to draw/stand with Bob** (not his turn)

**Expected:** HTTP 409 "NOT_YOUR_TURN" error

---

### Scenario 3: Enrollment Phase ✅

1. Create game (enrollment open)
2. **Try to draw card** before closing enrollment

**Expected:** HTTP 409 "NOT_YOUR_TURN" (enrollment must be closed)

---

### Scenario 4: Auto-Finish Logic ✅

1. Create game with 2 players
2. Close enrollment
3. **Player 1 stands**
4. Check game state → Still running
5. **Player 2 stands**

**Expected:** Game auto-finishes, game_finished = true

---

### Scenario 5: Invitation System ✅

1. Create game, enroll players
2. **Alice creates invitation** for external user
3. **List pending invitations** (as invitee)
4. **Accept invitation** → Auto-enrolls in game
5. Check enrolled players count

**Expected:** Invitee is enrolled, count increases

---

### Scenario 6: Stand After Bust ✅

1. Player draws cards until bust (points > 21)
2. **Try to stand** (should fail - already busted)

**Expected:** Error (player not active)

---

## Expected Test Results

### ✅ Success Indicators

- All HTTP requests return 2xx status codes (in happy path)
- Turn advances correctly after draw/stand
- Only current player can act
- Game auto-finishes when all players done
- Results show correct winner
- Invitation system works end-to-end

### ✅ Error Cases Work Correctly

- 409 NOT_YOUR_TURN when wrong player tries to act
- 410 ENROLLMENT_CLOSED when trying to enroll after closure
- 403 NOT_ENROLLED when non-enrolled user tries to invite
- 404 GAME_NOT_FOUND for invalid game_id

---

## Test Data

### Users
- `alice@test.com` / `password123` (creator)
- `bob@test.com` / `password123`
- `charlie@test.com` / `password123`
- `diana@test.com` / `password123` (invited)

### Typical Game IDs
- UUIDs generated on game creation
- Use response from create game endpoint

---

## Debugging

### Server Logs

Run with debug logging:
```powershell
$env:RUST_LOG="debug"
cargo run -p blackjack-api
```

Look for:
- `[INFO]` Game created, enrollment closed, turn advanced
- `[WARN]` Validation failures
- `[ERROR]` Unexpected errors

### Common Issues

**Issue:** Server not responding
- **Solution:** Check if port 8080 is free
- **Solution:** Restart server

**Issue:** 401 Unauthorized
- **Solution:** Check JWT token is valid and not expired
- **Solution:** Re-login to get fresh token

**Issue:** 404 Game Not Found
- **Solution:** Verify game_id is correct
- **Solution:** Game may have been cleaned up

**Issue:** 409 Not Your Turn
- **Solution:** Check game state to see whose turn it is
- **Solution:** Use correct player's token

---

## Performance Testing

### Load Test (Optional)

Create 10 players, max capacity:

```powershell
# Register 10 users
for ($i=1; $i -le 10; $i++) {
    # Create user player$i@test.com
    # Enroll in game
}
```

**Expected:** Game handles 10 players correctly

---

## Manual Verification Checklist

- [ ] Health check returns healthy status
- [ ] User registration works
- [ ] User login returns valid JWT
- [ ] Game creation sets enrollment timer
- [ ] Open games list shows new game
- [ ] Player enrollment increases count
- [ ] Invitation creation succeeds
- [ ] Pending invitations list shows invitation
- [ ] Close enrollment sets turn order
- [ ] Game state shows current player
- [ ] Draw card validates turn
- [ ] Draw card advances turn
- [ ] Stand marks player as standing
- [ ] Stand advances turn
- [ ] Wrong player gets NOT_YOUR_TURN error
- [ ] All players done triggers auto-finish
- [ ] Results show correct winner
- [ ] Tied players detected correctly
- [ ] All bust scenario handled

---

## Next Steps After Testing

If all tests pass:
1. ✅ Update Postman collection with new examples
2. ✅ Document any bugs found
3. ✅ Update API documentation
4. ✅ Prepare for merge to main branch
5. ✅ Consider Milestone 8 (Security Hardening)

---

## Quick Commands Reference

```powershell
# Start server
cargo run -p blackjack-api

# Run all tests
cargo test --workspace

# Check for issues
cargo clippy --workspace

# Build release
cargo build --release

# Run automated test script
.\docs\postman\test_phase2.ps1
```

---

**Happy Testing!** 🎉
