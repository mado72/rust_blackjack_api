# Blackjack API Architecture

**Status:** ✅ **FULLY IMPLEMENTED + ENHANCED SCORING**  
**Last Updated:** January 15, 2026  
**Tests:** 106 passing (60 integration tests in core)

## Overview

The Blackjack API is a turn-based multiplayer REST API built with Rust/Axum. It features user authentication, game invitation system, turn-based gameplay with automatic dealer play, and comprehensive per-player scoring.

**Key Features:**
- 🔐 User registration and JWT authentication
- 📨 Game invitation system with configurable timeouts
- 🎮 Turn-based card drawing with validation
- 🎲 **Automatic dealer play** when all players finish
- 📊 **Enhanced scoring** with individual outcomes (Won/Lost/Push/Busted)
- 🏆 Player state management (Active/Standing/Busted)
- ⚡ Automatic game completion
- 🎯 Real-time game state tracking

## Architecture Layers

### Core Layer (`blackjack-core`)

The core layer contains all business logic and domain models:

**User Management:**
- `User` struct with email and password_hash
- Placeholder password hashing (to be enhanced with Argon2)

**Invitation System:**
- `GameInvitation` struct with timeout support
- `InvitationStatus` enum (Pending, Accepted, Declined, Expired)
- Configurable timeouts (default: 300s, max: 3600s)
- Automatic expiration checking

**Game State:**
- `PlayerState` enum (Active, Standing, Busted)
- Turn order tracking (`turn_order: Vec<String>`)
- Current turn index (`current_turn_index: usize`)
- Game creator tracking (`creator_id: Uuid`)

**Turn Management:**
- `get_current_player()` - Returns current player's email
- `advance_turn()` - Moves to next active player
- `can_player_act()` - Validates player's turn
- `stand()` - Marks player as standing
- `check_auto_finish()` - Auto-finishes when all players done
- `add_player()` - Adds player from invitation acceptance

**Dealer Logic (NEW - January 15, 2026):**
- `play_dealer()` - Automatic dealer play
  - Draws cards until reaching 17+ points
  - Marks dealer as standing when finished (not busted)
  - Comprehensive logging at info and debug levels
  - Triggered automatically when all players finish
  - Cannot play after game finished
  - Handles empty deck scenario

**Enhanced Scoring (NEW - January 15, 2026):**
- `PlayerOutcome` enum: Won, Lost, Push, Busted
- `PlayerResult` struct:
  - `points`: Final score
  - `cards_count`: Number of cards
  - `busted`: Whether player busted
  - `outcome`: Individual outcome vs dealer
- Enhanced `GameResult`:
  - `player_results`: HashMap with detailed per-player outcomes
  - `dealer_points`: Dealer's final score
  - `dealer_busted`: Whether dealer busted
  - Backward-compatible with existing fields
- `calculate_results()` - Enhanced to populate detailed outcomes
  **Automatic dealer triggering** - When all players finish
- **Automatic game completion** - After dealer plays
- **Enhanced results** - Detailed per-player outcomesed for each player
  - Handles all tie scenarios (multiple winners, all push, all lose)
  - Maintains legacy winner/tied_players logic

### Service Layer (`blackjack-service`)

The service layer orchestrates business logic:

**UserService:**
- User registration
- User login with credential verification
- User lookup by ID or email

**InvitationService:**
- Create invitations with custom timeout
- Accept/decline invitations
- Get pending invitations (auto-filters expired)
- Cleanup expired invitations
- Timeout validation against maximum

**GameService:**
- `create_game(creator_id, emails)` - Requires creator ID
- `stand(game_id, email)` - Player stands
- `add_player_to_game()` - Add player from invitation
- `is_game_creator()` - Check creator permission
- Turn-based draw card validation
- Automatic game completion logic

**Configuration:**
- `InvitationConfig` with default and max timeouts
- Environment variable support

### API Layer (`blackjack-api`)

The API layer exposes HTTP endpoints:

**AppState:**
- `user_service: Arc<UserService>`
- `invitation_service: Arc<InvitationService>`
- `game_service: Arc<GameService>`

**JWT Claims:** 
- `user_id: String` - Unique user identifier
- `email: String` - User email
- `exp: usize` - Token expiration (24 hours default)

**Middleware:**
- JWT authentication
- Rate limiting per user
- Security headers

## API Endpoints

### Authentication (2 endpoints)
- `POST /api/v1/auth/register` - Register new user
- `POST /api/v1/auth/login` - User authentication

### Health Checks (2 endpoints)
- `GET /health` - Server health status
- `GET /health/ready` - Component readiness

### Game Management (4 endpoints) and dealer state
- `POST /api/v1/games/:id/finish` - Finish game manually
- `GET /api/v1/games/:id/results` - **View enhanced results with per-player outcomes**turn info
- `POST /api/v1/games/:id/finish` - Finish game manually
- `GET /api/v1/games/:id/results` - View results

### Gameplay (3 endpoints)
- `POST /api/v1/games/:id/draw` - Draw card (turn validated)
- `PUT /api/v1/games/:id/ace` - Change Ace value
- `POST /api/v1/games/:id/stand` - Player stands

### Invitations (5 endpoints)
- `POST /api/v1/games/:id/invitations` - Create invitation
- `GET /api/v1/invitations/pending` - Get pending invitations
- `POST /api/v1/invitations/:id/accept` - Accept invitation
- `POST /api/v1/invitations/:id/decline` - Decline invitation

**Total: 16 HTTP endpoints**

**Total: 16 HTTP endpoints**

## Configuration

### Environment Variables

```bash
# Server
BLACKJACK_SERVER_HOST=127.0.0.1
BLACKJACK_SERVER_PORT=8080

# JWT
BLACKJACK_JWT_SECRET=your-secret-key
BLACKJACK_JWT_EXPIRATION_HOURS=24

# Rate Limiting
BLACKJACK_RATE_LIMIT_REQUESTS_PER_MINUTE=20

# Invitation timeouts
BLACKJACK_INVITATIONS_DEFAULT_TIMEOUT_SECONDS=300
BLACKJACK_INVITATIONS_MAX_TIMEOUT_SECONDS=3600
```

### config.toml

```toml
[server]
host = "127.0.0.1"
port = 8080

[jwt]
expiration_hours = 24

[rate_limit]
requests_per_minute = 20

[invitations]
default_timeout_seconds = 300  # 5 minutes default
max_timeout_seconds = 3600     # 1 hour maximum
```

## Game Flow

### 1. User Registration & Login
```
POST /api/v1/auth/register → Register users
POST /api/v1/auth/login → Get JWT token
```

### 2. Game Creation
```
POST /api/v1/games → Creator starts game with initial players
```

### 3. Player Invitation (Optional)
```
POST /api/v1/games/:id/invitations → Send invitations
GET /api/v1/invitations/pending → Check pending invitations
POST /api/v1/invitations/:id/accept → Accept invitation
```

### 4. Turn-Based Gameplay
```
GET /api/v1/games/:id → Check whose turn it is
POST /api/v1/games/:id/draw → Draw card (only on your turn)
POST /api/v1/games/:id/stand → Stand when done
```

**Automatic Dealer Play:**
- When last player stands, dealer automatically plays
- Dealer draws until reaching 17+ points
- Game finishes automatically after dealer completes

### 5. Game Completion & Results
```
Game auto-finishes when all players stand/bust (triggers dealer)
GET /api/v1/games/:id/results → View detailed results

Results include:
- Individual outcomes for each player (Won/Lost/Push/Busted)
- Dealer final state (points, busted)
- Overall winner or tie determination
- Backward-com60 tests (integration)
- Service layer: 13 tests  
- API layer: 20 tests
- CLI: 13 tests

**Total: 106 tests passing** ✅

**Test Coverage:**
- ✅ Dealer automatic play (11 tests)
- ✅ Enhanced scoring with all outcomes (12 tests)
- ✅ Turn management and validation
- ✅ Enrollment and invitations
- ✅ All tie/draw scenarios
```bash
cargo test --workspace
```Dealer Play**: Dealer automatically plays when all players finish (stand/bust)
3. **Enhanced Scoring**: Individual outcomes (Won/Lost/Push/Busted) for each player
4. **Automatic Completion**: Game finishes after dealer completes play
5. **Invitation Expiration**: Configurable timeouts with automatic cleanup
6. **JWT Authentication**: 24-hour tokens with user_id and email claims
7. **Rate Limiting**: Per-user rate limiting to prevent abuse
8. **Placeholder Authentication**: Simple password hashing (to be enhanced with Argon2)
9. **Backward Compatibility**: New scoring fields added while preserving existing API
- API layer: 16 tests
- CLI: 13 tests

**Total: 60 tests passing** ✅

### Manual Testing
- Postman collection: `Blackjack_API.postman_collection.json`
- VS Code REST Client: `api_tests.http`
- PowerShell script: `test_api.ps1`

## Technical Decisions
Completed (January 15, 2026) ✅
- ✅ Automatic dealer play with comprehensive logging
- ✅ Enhanced scoring with per-player outcomes
- ✅ All tie/draw scenario handling
- ✅ Detailed game results endpoint

### High Priority
- Argon2 password hashing
- Database persistence (PostgreSQL)
- Integration tests for complete workflows
- WebSocket support for real-time updates
- Two-player concurrent testing (Postman environments created) with user_id and email claims
5. **Rate Limiting**: Per-user rate limiting to prevent abuse
6. **Placeholder Authentication**: Simple password hashing (to be enhanced with Argon2)

## Future Enhancements

### High Priority
- Argon2 password hashing
- Database persistence (PostgreSQL)
- Integration tests for complete workflows
- WebSocket support for real-time updates

### Medium Priority
- Admin endpoints
- Game history and statistics
- Enhanced error handling
- Metrics and monitoring

### Low Priority
- Multi-deck support
- TournaPlayerOutcome, PlayerResult (NEW)
│   ├── Turn management
│   ├── Dealer logic with auto-play (NEW)
│   ├── Enhanced scoring system (NEW)
│   └── Auto-finish logic
│
├── blackjack-service/      # Service orchestration
│   ├── UserService
│   ├── GameService (with dealer triggering)
```
crates/
├── blackjack-core/         # Domain models and business logic
│   ├── User, Game, Invitation structs
│   ├── Turn management
│   └── Auto-finish logic
│
├── blackjack-service/      # Service orchestration
│   ├── UserService
│   ├── GameService
│   ├── InvitationService
│   └── Configuration
│
├── blackjack-api/          # HTTP API layer
│   ├── handlers.rs         # 16 endpoint handlers
│   ├── main.rs             # Route registration
│   ├── auth.rs             # JWT middleware
│   ├── middleware.rs       # Rate limiting, security
│   └── config.rs           # Configuration loading
│
└── blackjack-cli/          # CLI interface (optional)
```

## Security

- JWT-based authentication
- Rate limiting per user
- Password hashing (placeholder, upgrade to Argon2 planned)
- ITWO_PLAYER_TESTING_GUIDE.md](TWO_PLAYER_TESTING_GUIDE.md) - Concurrent multi-player testing
- [DEALER_CURL_EXAMPLES.md](DEALER_CURL_EXAMPLES.md) - Complete flow with dealer auto-play
- [QUICK_REFERENCE.md](QUICK_REFERENCE.md)
- [POSTMAN_GUIDE.md](POSTMAN_GUIDE.md)
- [CURL_EXAMPLES.md](CURL_EXAMPLES.md)
- [DEALER_IMPLEMENTATION.md](../DEALER_IMPLEMENTATION.md) - Dealer logic documentation
## Performance Considerations

- In-memory storage (for now)
- Arc/Mutex for thread-safe shared state
- Efficient turn advancement (skips inactive players)
- Automatic cleanup of expired invitations

---

**For detailed testing instructions, see:**
- [QUICK_REFERENCE.md](QUICK_REFERENCE.md)
- [POSTMAN_GUIDE.md](POSTMAN_GUIDE.md)
- [CURL_EXAMPLES.md](CURL_EXAMPLES.md)
