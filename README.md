# Blackjack Multi-Player Backend System

A production-ready REST API backend for multi-player Blackjack card games, built with Rust using Axum, featuring JWT authentication, Argon2id password hashing, role-based access control, rate limiting, structured logging, and comprehensive security hardening.

## Overview

This project provides a complete backend system for managing multi-player Blackjack games (1-10 players per game) with:

- **RESTful API**: Versioned endpoints under `/api/v1` with comprehensive documentation
- **JWT Authentication**: Secure player authentication per game session
- **Security Hardening (M8)**: Argon2id password hashing (OWASP parameters), RBAC, security headers
- **User Management (M7-M8)**: User registration, login, password changes, account activation/deactivation
- **Access Control (M8)**: Role-based permissions (Creator, Player, Spectator)
- **Game Management (M8)**: Kick players, view participants with roles
- **Turn-Based Gameplay (M7)**: Ordered turns, automatic advancement, smart turn skipping
- **Game Invitations (M7)**: Invite system with configurable timeouts and status tracking
- **Rate Limiting**: Per-user request throttling using sliding window algorithm
- **Real-time Ready**: WebSocket blueprint for future real-time notifications
- **Observability**: Structured logging with tracing, health checks, metrics-ready
- **Production-Grade**: External configuration, CORS support, graceful error handling
- **Multi-player Support**: 1-10 players per game with independent state management
- **Flexible Gameplay**: Dynamic Ace values (1 or 11), ordered card history, bust detection
- **Comprehensive Testing**: 167 tests passing (unit, integration, API, service)

## Game Rules

### Card Values
- **Numbered cards (2-9)**: Face value
- **10, Jack, Queen, King**: 10 points each
- **Ace**: 1 point (can be changed to 11 points at player's discretion)

### Gameplay Flow (Milestone 7 - Game Lobby System)

**Phase 1: Game Creation & Enrollment**
1. **Creator** creates a game with optional enrollment timeout (default: 300s)
2. **Players** can:
   - Browse open games via `/api/v1/games/open`
   - Self-enroll in games with available slots (max 10 players)
   - Receive and accept game invitations from enrolled players
3. **Enrollment Period**:
   - Players can join until timeout expires OR creator manually closes enrollment
   - Game accepts players up to maximum capacity (10 players)
4. **Creator** closes enrollment to start the game
   - Turn order is randomized when enrollment closes

**Phase 2: Turn-Based Gameplay**
1. Players take **ordered turns** (enforced by server)
2. On each turn, the current player can:
   - Draw a card from the shared deck
   - Change Ace values (1 ↔ 11 points)
   - Stand (end their turn)
3. After drawing a card:
   - Server validates it's the player's turn (returns 409 if not)
   - If player busts (>21), their turn automatically advances
   - If player stands, turn advances to next active player
4. **Auto-finish**: Game automatically finishes when all players have stood or busted
5. Winner is determined based on highest score ≤21

### Winning Conditions
- **Single Winner**: Player with highest score ≤21
- **Tie**: Multiple players with the same highest score ≤21
- **No Winner**: All players exceeded 21 points (all bust)

## Architecture

This project uses a **workspace-based architecture** with clear separation of concerns:

```
rust_blackjack/
├── Cargo.toml                    # Workspace manifest
├── Dockerfile                    # Multi-stage Docker build
├── .dockerignore
├── .github/
│   └── workflows/
│       └── ci.yml               # CI/CD pipeline
├── crates/
│   ├── blackjack-core/          # Domain logic (Game, Card, Player)
│   │   ├── src/lib.rs
│   │   └── tests/integration_tests.rs
│   ├── blackjack-service/       # Business logic layer
│   │   ├── src/lib.rs
│   │   ├── migrations/          # Future SQLite migrations
│   │   └── tests/service_tests.rs
│   ├── blackjack-api/           # REST API (Axum)
│   │   ├── src/
│   │   │   ├── main.rs          # Server entry point
│   │   │   ├── lib.rs
│   │   │   ├── auth.rs          # JWT authentication
│   │   │   ├── config.rs        # Configuration management
│   │   │   ├── error.rs         # Standardized errors
│   │   │   ├── handlers.rs      # HTTP request handlers
│   │   │   ├── middleware.rs    # Auth, rate limit, deprecation
│   │   │   ├── rate_limiter.rs  # Sliding window rate limiter
│   │   │   └── websocket.rs     # WebSocket blueprint (future)
│   │   ├── config.toml          # Default configuration
│   │   └── tests/api_tests.rs
│   └── blackjack-cli/           # Original CLI version (preserved)
│       └── src/main.rs
├── docs/
│   ├── PRD.md                   # Product Requirements Document
│   └── postman/                 # API testing resources
│       ├── README.md            # Testing guide overview
│       ├── Blackjack_API.postman_collection.json
│       ├── Blackjack_API_Local.postman_environment.json
│       ├── POSTMAN_GUIDE.md     # Complete Postman tutorial
│       ├── QUICK_REFERENCE.md   # Quick reference guide
│       ├── CURL_EXAMPLES.md     # cURL command examples
│       ├── API_TESTING_INDEX.md # Complete testing index
│       ├── api_tests.http       # VS Code REST Client file
│       └── test_api.ps1         # PowerShell test script
└── README.md                    # This file
```

### Layer Responsibilities

- **blackjack-core**: Pure domain logic, no external dependencies
- **blackjack-service**: Orchestration, concurrency, validation
- **blackjack-api**: HTTP, authentication, rate limiting, serialization
- **blackjack-cli**: Original terminal-based game (preserved for reference)

## Quick Start

### Prerequisites

- Rust 1.75 or later
- Docker (optional, for containerized deployment)

### Development Setup

```bash
# Clone the repository
git clone <repository-url>
cd rust_blackjack

# Build the workspace
cargo build --workspace

# Run tests
cargo test --workspace

# Run the API server (development mode)
RUST_LOG=debug cargo run -p blackjack-api

# Run the original CLI game
cargo run -p blackjack-cli
```

The API server will start on `http://127.0.0.1:8080` by default.

## Testing the API

Multiple options are available for testing the API endpoints:

### Option 1: Postman Collection (Recommended)

Import the pre-configured Postman collection with all endpoints:

1. Open Postman
2. Click **Import** → Select files
3. Import both:
   - `docs/postman/Blackjack_API.postman_collection.json` - Complete collection
   - `docs/postman/Blackjack_API_Local.postman_environment.json` - Environment variables
4. Select **Blackjack API - Local** environment
5. Start with **Create Game** → **Login** → **Draw Card**

**Features:**
- ✅ Automatic token management (JWT saved automatically)
- ✅ Automatic game_id management
- ✅ Pre-configured requests with examples
- ✅ Test scripts with console logging
- ✅ Full documentation in each request

📖 **See [docs/postman/POSTMAN_GUIDE.md](docs/postman/POSTMAN_GUIDE.md) for detailed instructions**

### Option 2: VS Code REST Client

Use the `.http` file for quick testing in VS Code:

1. Install the **REST Client** extension
2. Open `docs/postman/api_tests.http`
3. Click "Send Request" above each request
4. Modify variables at the top of the file

### Option 3: PowerShell Script (Automated Testing)

Run the complete test suite automatically:

```powershell
# Make sure the server is running first
.\docs\postman\test_api.ps1
```

This script will:
- ✅ Test all endpoints in order
- ✅ Save variables automatically
- ✅ Show detailed colored output
- ✅ Test error scenarios
- ✅ Provide a complete summary

### Option 4: cURL Commands

For command-line testing, see `docs/postman/CURL_EXAMPLES.md` with ready-to-use examples:

```bash
# Health check
curl http://localhost:8080/health | jq '.'

# Create game (save the game_id)
curl -X POST http://localhost:8080/api/v1/games \
  -H "Content-Type: application/json" \
  -d '{"emails":["player1@example.com"]}'

# More examples in docs/postman/CURL_EXAMPLES.md...
```

### API Testing Files

All testing resources are located in the `docs/postman/` directory:

| File | Purpose | Best For |
|------|---------|----------|
| `docs/postman/Blackjack_API.postman_collection.json` | Postman collection | Interactive testing, documentation |
| `docs/postman/Blackjack_API_Local.postman_environment.json` | Postman environment | Variable management |
| `docs/postman/POSTMAN_GUIDE.md` | Complete Postman guide | Learning the API flow |
| `docs/postman/api_tests.http` | REST Client file | Quick VS Code testing |
| `docs/postman/test_api.ps1` | PowerShell test script | Automated full suite |
| `docs/postman/CURL_EXAMPLES.md` | cURL examples | Command-line reference |
| `docs/postman/QUICK_REFERENCE.md` | Quick reference guide | Fast lookup |
| `docs/postman/API_TESTING_INDEX.md` | Complete testing index | Navigation hub |

### Quick Test Flow

1. **Start the server**: `cargo run -p blackjack-api`
2. **Health check**: Verify server is running
3. **Create game**: Get a `game_id`
4. **Login**: Get a JWT token
5. **Play**: Draw cards, change Ace values
6. **Finish**: End game and see results

All testing tools follow this same flow with automatic variable management!

## Configuration

Configuration is loaded from `crates/blackjack-api/config.toml` and can be overridden with environment variables prefixed with `BLACKJACK_`.

### config.toml

```toml
[server]
host = "127.0.0.1"
port = 8080

[cors]
allowed_origins = ["http://localhost:3000"]

[jwt]
secret = "dev-secret-key-change-in-production"
expiration_hours = 24

[rate_limit]
requests_per_minute = 10

[api]
version_deprecation_months = 6
```

### Environment Variables

Environment variables take precedence over `config.toml`:

```bash
# Server configuration
export BLACKJACK_SERVER_HOST=0.0.0.0
export BLACKJACK_SERVER_PORT=3000

# JWT configuration
export BLACKJACK_JWT_SECRET=your-production-secret-key
export BLACKJACK_JWT_EXPIRATION_HOURS=12

# Rate limiting
export BLACKJACK_RATE_LIMIT_REQUESTS_PER_MINUTE=20

# Logging (uses RUST_LOG standard)
export RUST_LOG=info
# or for detailed debugging:
export RUST_LOG=debug
```

### .env.example

For local development, copy `.env.example` to `.env`:

```bash
BLACKJACK_JWT_SECRET=dev-secret-change-in-production
BLACKJACK_SERVER_PORT=8080
RUST_LOG=debug
```

## API Reference

### Base URL

```
http://localhost:8080
```

All API endpoints are versioned under `/api/v1`.

### API Endpoint Categories

- **Health Checks**: `/health`, `/health/ready`
- **Authentication**: `/api/v1/auth/register`, `/api/v1/auth/login`
- **Game Lifecycle (M7)**: Create, browse open games, enroll, close enrollment
- **Invitations (M7)**: Create, list pending, accept, decline
- **Gameplay (M7)**: Turn-based draw, stand, game state
- **Game Results**: Finish game, get results

### Health Check Endpoints

#### GET /health

Basic health check.

**Response (200 OK):**
```json
{
  "status": "healthy",
  "uptime_seconds": 3600,
  "version": "0.1.0"
}
```

#### GET /health/ready

Readiness check for orchestration systems (Kubernetes, etc.).

**Response (200 OK):**
```json
{
  "ready": true,
  "checks": {
    "memory": "ok",
    "config": "loaded",
    "future_sqlite": "pending",
    "future_metrics": "pending"
  }
}
```

### Authentication

#### Password Requirements (Milestone 8 Security)

All passwords must meet the following complexity requirements:
- **Minimum length**: 8 characters
- **Must contain**:
  - At least one uppercase letter (A-Z)
  - At least one lowercase letter (a-z)
  - At least one digit (0-9)
  - At least one special character (!@#$%^&*)

**Valid password examples:**
- `MyP@ssw0rd`
- `Secure#Pass123`
- `Test!User2024`

#### POST /api/v1/auth/register

Register a new user account. (Milestone 7)

**Request:**
```json
{
  "email": "newplayer@example.com",
  "password": "Secure#Pass123"
}
```

**Response (200 OK):**
```json
{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "newplayer@example.com",
  "message": "User registered successfully"
}
```

**Errors:**
- `400` - Invalid email format
- `400` - Weak password (doesn't meet complexity requirements)
  - Error includes specific requirements that weren't met
- `409` - Email already registered

#### POST /api/v1/auth/login

Login with existing user credentials. (Milestone 7)

**Request:**
```json
{
  "email": "player1@example.com",
  "password": "Secure#Pass123"
}
```

**Response (200 OK):**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_in": 86400
}
```

**Errors:**
- `401` - Invalid credentials (wrong email or password)
- `403` - Account inactive (account has been deactivated)
- `404` - User not found

#### POST /api/v1/auth/change-password

Change user password. **Requires authentication.** (Milestone 8)

**Headers:**
```
Authorization: Bearer <jwt_token>
```

**Request:**
```json
{
  "old_password": "OldSecure#Pass123",
  "new_password": "NewSecure#Pass456"
}
```

**Response (200 OK):**
```json
{
  "message": "Password changed successfully"
}
```

**Errors:**
- `401` - Unauthorized (missing or invalid token)
- `401` - Invalid old password
- `400` - New password doesn't meet complexity requirements
- `404` - User not found

### Game Management

#### POST /api/v1/games

Create a new game with enrollment system. **Requires authentication.** (Milestone 7)

**Headers:**
```
Authorization: Bearer <jwt_token>
```

**Request:**
```json
{
  "enrollment_timeout_seconds": 300
}
```

**Response (200 OK):**
```json
{
  "game_id": "550e8400-e29b-41d4-a716-446655440000",
  "creator_id": "user-uuid",
  "message": "Game created successfully",
  "player_count": 1,
  "enrollment_closes_at": "2026-01-14T12:05:00Z"
}
```

**Errors:**
- `401` - Unauthorized (missing or invalid token)
- `400` - Invalid timeout value

#### GET /api/v1/games/open

Get list of games accepting enrollment. **Requires authentication.** (Milestone 7)

**Headers:**
```
Authorization: Bearer <jwt_token>
```

**Response (200 OK):**
```json
{
  "games": [
    {
      "game_id": "550e8400-e29b-41d4-a716-446655440000",
      "creator_id": "user-uuid",
      "enrolled_count": 3,
      "max_players": 10,
      "enrollment_timeout_seconds": 300,
      "time_remaining_seconds": 245,
      "enrollment_closes_at": "2026-01-14T12:05:00Z"
    }
  ],
  "count": 1
}
```

**Errors:**
- `401` - Unauthorized

#### POST /api/v1/games/:game_id/enroll

Enroll the authenticated user in a game. **Requires authentication.** (Milestone 7)

**Headers:**
```
Authorization: Bearer <jwt_token>
```

**Request:**
```json
{
  "email": "player2@example.com"
}
```

**Response (200 OK):**
```json
{
  "game_id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "player2@example.com",
  "message": "Player enrolled successfully",
  "enrolled_count": 2
}
```

**Errors:**
- `401` - Unauthorized
- `404` - Game not found
- `409` - Game is full (10 players max)
- `410` - Enrollment period has closed

#### POST /api/v1/games/:game_id/close-enrollment

Close enrollment and start the game. **Only creator can close.** (Milestone 7)

**Headers:**
```
Authorization: Bearer <jwt_token>
```

**Request:**
```json
{}
```

**Response (200 OK):**
```json
{
  "game_id": "550e8400-e29b-41d4-a716-446655440000",
  "message": "Enrollment closed, game ready to start",
  "turn_order": ["player1@example.com", "player2@example.com", "player3@example.com"],
  "player_count": 3
}
```

**Errors:**
- `401` - Unauthorized
- `403` - Only game creator can close enrollment
- `404` - Game not found

#### DELETE /api/v1/games/:game_id/players/:player_id

Kick a player from the game. **Only creator can kick players.** (Milestone 8)

**Headers:**
```
Authorization: Bearer <jwt_token>
```

**Path Parameters:**
- `game_id` - The game UUID
- `player_id` - The player's user UUID to kick

**Response (200 OK):**
```json
{
  "game_id": "550e8400-e29b-41d4-a716-446655440000",
  "player_id": "player-uuid",
  "player_email": "player2@example.com",
  "message": "Player player2@example.com kicked successfully"
}
```

**Errors:**
- `401` - Unauthorized
- `403` - Only game creator can kick players
- `403` - Cannot kick the game creator
- `404` - Game not found
- `404` - Player not found in game
- `409` - Can only kick players during enrollment phase

#### GET /api/v1/games/:game_id/participants

Get all participants in a game. **Requires authentication.** (Milestone 8)

**Headers:**
```
Authorization: Bearer <jwt_token>
```

**Response (200 OK):**
```json
{
  "game_id": "550e8400-e29b-41d4-a716-446655440000",
  "participants": [
    {
      "user_id": "550e8400-e29b-41d4-a716-446655440000",
      "email": "creator@example.com",
      "role": "Creator"
    },
    {
      "user_id": "660e8400-e29b-41d4-a716-446655440001",
      "email": "player1@example.com",
      "role": "Player"
    }
  ],
  "count": 2
}
```

**Errors:**
- `401` - Unauthorized
- `404` - Game not found

### Game Invitations (Milestone 7)

#### POST /api/v1/games/:game_id/invitations

Send an invitation to another user to join the game. **Requires authentication.**

**Headers:**
```
Authorization: Bearer <jwt_token>
```

**Request:**
```json
{
  "invitee_email": "friend@example.com"
}
```

**Response (200 OK):**
```json
{
  "invitation_id": "invite-uuid",
  "game_id": "550e8400-e29b-41d4-a716-446655440000",
  "inviter_email": "player1@example.com",
  "invitee_email": "friend@example.com",
  "status": "pending",
  "expires_at": "2026-01-14T12:05:00Z",
  "message": "Invitation sent successfully"
}
```

**Errors:**
- `401` - Unauthorized
- `403` - You must be enrolled in the game to send invitations
- `404` - Game not found
- `410` - Enrollment period has closed

#### GET /api/v1/invitations/pending

Get all pending invitations for the authenticated user. **Requires authentication.**

**Headers:**
```
Authorization: Bearer <jwt_token>
```

**Response (200 OK):**
```json
{
  "invitations": [
    {
      "invitation_id": "invite-uuid",
      "game_id": "550e8400-e29b-41d4-a716-446655440000",
      "inviter_email": "player1@example.com",
      "status": "pending",
      "expires_at": "2026-01-14T12:05:00Z",
      "created_at": "2026-01-14T12:00:00Z"
    }
  ],
  "count": 1
}
```

**Errors:**
- `401` - Unauthorized

#### POST /api/v1/invitations/:invitation_id/accept

Accept a game invitation. **Requires authentication.**

**Headers:**
```
Authorization: Bearer <jwt_token>
```

**Response (200 OK):**
```json
{
  "message": "Invitation accepted, you are now enrolled in the game",
  "game_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Errors:**
- `401` - Unauthorized
- `404` - Invitation not found
- `409` - Game is full
- `410` - Invitation has expired

#### POST /api/v1/invitations/:invitation_id/decline

Decline a game invitation. **Requires authentication.**

**Headers:**
```
Authorization: Bearer <jwt_token>
```

**Response (200 OK):**
```json
{
  "message": "Invitation declined"
}
```

**Errors:**
- `401` - Unauthorized
- `404` - Invitation not found

### Gameplay Endpoints (Turn-Based - Milestone 7)

#### GET /api/v1/games/:game_id

Get current game state. **Requires authentication.**

**Headers:**
```
Authorization: Bearer <jwt_token>
```

**Response (200 OK):**
```json
{
  "game_id": "550e8400-e29b-41d4-a716-446655440000",
  "enrollment_open": false,
  "current_turn_player": "player2@example.com",
  "turn_order": ["player1@example.com", "player2@example.com", "player3@example.com"],
  "players": {
    "player1@example.com": {
      "points": 18,
      "state": "Standing",
      "cards_history": [
        {
          "id": "card-uuid-1",
          "name": "King",
          "value": 10,
          "suit": "Hearts"
        },
        {
          "id": "card-uuid-2",
          "name": "8",
          "value": 8,
          "suit": "Diamonds"
        }
      ],
      "busted": false
    },
    "player2@example.com": {
      "points": 15,
      "state": "Active",
      "cards_history": [...],
      "busted": false
    }
  },
  "cards_in_deck": 46,
  "finished": false
}
```

**Errors:**
- `401` - Unauthorized (missing or invalid token)
- `404` - Game not found

#### POST /api/v1/games/:game_id/draw

Draw a card for the authenticated player. **Turn-based - validates current turn.** (Milestone 7)

**Headers:**
```
Authorization: Bearer <jwt_token>
```

**Response (200 OK):**
```json
{
  "card": {
    "id": "card-uuid",
    "name": "Ace",
    "value": 11,
    "suit": "Spades"
  },
  "current_points": 21,
  "busted": false,
  "cards_remaining": 45,
  "cards_history": [
    {
      "id": "card-uuid-1",
      "name": "King",
      "value": 10,
      "suit": "Hearts"
    },
    {
      "id": "card-uuid-2",
      "name": "Ace",
      "value": 11,
      "suit": "Spades"
    }
  ],
  "is_finished": false,
  "next_player": "player3@example.com"
}
```

**Errors:**
- `401` - Unauthorized
- `404` - Game or player not found
- `409` - Not your turn (NOT_YOUR_TURN) OR enrollment still open (ENROLLMENT_NOT_CLOSED)
- `410` - Deck is empty

#### POST /api/v1/games/:game_id/stand

Stand (end turn without drawing). **Turn-based - validates current turn.** (Milestone 7)

**Headers:**
```
Authorization: Bearer <jwt_token>
```

**Response (200 OK):**
```json
{
  "message": "Player stood successfully",
  "current_points": 18,
  "is_finished": false,
  "next_player": "player3@example.com"
}
```

**Auto-finish Response (200 OK - when all players done):**
```json
{
  "message": "Player stood successfully",
  "current_points": 18,
  "is_finished": true,
  "winner": "player1@example.com",
  "results": {
    "winner": "player1@example.com",
    "tied_players": [],
    "highest_score": 21,
    "all_players": {
      "player1@example.com": {
        "points": 21,
        "cards_count": 2,
        "busted": false
      },
      "player2@example.com": {
        "points": 18,
        "cards_count": 3,
        "busted": false
      }
    }
  }
}
```

**Errors:**
- `401` - Unauthorized
- `404` - Game not found
- `409` - Not your turn (NOT_YOUR_TURN)
- `410` - Enrollment still open

#### PUT /api/v1/games/:game_id/ace

Change an Ace value between 1 and 11. **Requires authentication.**

**Headers:**
```
Authorization: Bearer <jwt_token>
```

**Request:**
```json
{
  "card_id": "card-uuid",
  "as_eleven": false
}
```

**Response (200 OK):**
```json
{
  "points": 11,
  "busted": false
}
```

**Errors:**
- `401` - Unauthorized
- `403` - Game already finished
- `404` - Game, player, or card not found

#### POST /api/v1/games/:game_id/finish

Finish the game and calculate results. **Requires authentication.**

**Headers:**
```
Authorization: Bearer <jwt_token>
```

**Response (200 OK):**
```json
{
  "winner": "player1@example.com",
  "tied_players": [],
  "highest_score": 21,
  "all_players": {
    "player1@example.com": {
      "points": 21,
      "cards_count": 2,
      "busted": false
    },
    "player2@example.com": {
      "points": 19,
      "cards_count": 3,
      "busted": false
    }
  }
}
```

**Errors:**
- `401` - Unauthorized
- `404` - Game not found
- `409` - Game already finished

#### GET /api/v1/games/:game_id/results

Get results of a finished game. **Requires authentication.**

**Headers:**
```
Authorization: Bearer <jwt_token>
```

**Response:** Same as POST /api/v1/games/:game_id/finish

**Errors:**
- `401` - Unauthorized
- `404` - Game not found
- `409` - Game not finished yet

## Complete API Flow Example

Here's a complete example using curl to play a game:

```bash
# 1. Create a new game
GAME_RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/games \
  -H "Content-Type: application/json" \
  -d '{
    "emails": ["player1@example.com", "player2@example.com"]
  }')

GAME_ID=$(echo $GAME_RESPONSE | jq -r '.game_id')
echo "Game created: $GAME_ID"

# 2. Login as player1
TOKEN1=$(curl -s -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d "{
    \"email\": \"player1@example.com\",
    \"game_id\": \"$GAME_ID\"
  }" | jq -r '.token')

echo "Player1 authenticated"

# 3. Login as player2
TOKEN2=$(curl -s -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d "{
    \"email\": \"player2@example.com\",
    \"game_id\": \"$GAME_ID\"
  }" | jq -r '.token')

echo "Player2 authenticated"

# 4. Player1 draws a card
curl -s -X POST "http://localhost:8080/api/v1/games/$GAME_ID/draw" \
  -H "Authorization: Bearer $TOKEN1" | jq

# 5. Player1 draws another card
DRAW2=$(curl -s -X POST "http://localhost:8080/api/v1/games/$GAME_ID/draw" \
  -H "Authorization: Bearer $TOKEN1")

echo "$DRAW2" | jq

# 6. If player1 got an Ace, change its value
ACE_ID=$(echo "$DRAW2" | jq -r '.card | select(.name == "A") | .id')
if [ "$ACE_ID" != "null" ] && [ "$ACE_ID" != "" ]; then
  curl -s -X PUT "http://localhost:8080/api/v1/games/$GAME_ID/ace" \
    -H "Authorization: Bearer $TOKEN1" \
    -H "Content-Type: application/json" \
    -d "{
      \"card_id\": \"$ACE_ID\",
      \"as_eleven\": false
    }" | jq
fi

# 7. Player2 draws cards
curl -s -X POST "http://localhost:8080/api/v1/games/$GAME_ID/draw" \
  -H "Authorization: Bearer $TOKEN2" | jq

curl -s -X POST "http://localhost:8080/api/v1/games/$GAME_ID/draw" \
  -H "Authorization: Bearer $TOKEN2" | jq

# 8. View game state
curl -s "http://localhost:8080/api/v1/games/$GAME_ID" \
  -H "Authorization: Bearer $TOKEN1" | jq

# 9. Finish the game
curl -s -X POST "http://localhost:8080/api/v1/games/$GAME_ID/finish" \
  -H "Authorization: Bearer $TOKEN1" | jq

# 10. Get final results
curl -s "http://localhost:8080/api/v1/games/$GAME_ID/results" \
  -H "Authorization: Bearer $TOKEN1" | jq
```

## Milestone 7: Turn-Based Gameplay and User Management

**Status**: ✅ COMPLETE (January 14, 2026)

The M7 implementation introduces turn-based multiplayer gameplay with user management and game invitations.

### Phase 1: Enrollment Endpoints - COMPLETE ✅

**Completed (Jan 10, 2026):**
- ✅ `POST /api/v1/games` - Create game with enrollment timeout
- ✅ `GET /api/v1/games/open` - List games in enrollment phase  
- ✅ `POST /api/v1/games/:game_id/enroll` - Enroll player in game
- ✅ `POST /api/v1/games/:game_id/close-enrollment` - Close enrollment and initialize turns

### Phase 2: Invitation & Gameplay Endpoints - COMPLETE ✅

**Completed (Jan 14, 2026):**
- ✅ `POST /api/v1/games/:game_id/invitations` - Send game invitations
- ✅ `GET /api/v1/invitations/pending` - List pending invitations
- ✅ `POST /api/v1/invitations/:id/accept` - Accept invitation
- ✅ `POST /api/v1/invitations/:id/decline` - Decline invitation
- ✅ `POST /api/v1/games/:game_id/stand` - Stand and advance turn

### Phase 3: Turn Management & State - COMPLETE ✅

**Completed (Jan 14, 2026):**
- ✅ Turn order system with automatic advancement
- ✅ Player state tracking (standing, busted, active)
- ✅ Smart turn skipping for inactive players
- ✅ Auto-finish when all players complete turns

## Milestone 8: Security Hardening and Enhanced User Management

**Status**: ✅ COMPLETE (January 15, 2026)

The M8 implementation adds enterprise-grade security features and enhanced user management capabilities.

### Security Features - COMPLETE ✅

**Completed (Jan 15, 2026):**
- ✅ Argon2id password hashing (OWASP parameters: 19 MiB memory, 2 iterations)
- ✅ Password complexity validation (8+ chars, uppercase, lowercase, digit, special)
- ✅ Email validation (RFC 5322 compliant)
- ✅ Constant-time password verification (timing attack protection)
- ✅ Security headers middleware (CSP, HSTS, X-Frame-Options, X-Content-Type-Options)

### Access Control - COMPLETE ✅

**Completed (Jan 15, 2026):**
- ✅ Role-Based Access Control (RBAC) with GameRole enum
- ✅ GamePermission system (5 permissions)
- ✅ Permission checking at service layer
- ✅ Participant tracking with roles (Creator, Player, Spectator)

### User Account Management - COMPLETE ✅

**Completed (Jan 15, 2026):**
- ✅ `POST /api/v1/auth/change-password` - Change user password
- ✅ Account activation/deactivation
- ✅ Login tracking (last_login timestamp)
- ✅ Enhanced User model with security fields

### Game Management - COMPLETE ✅

**Completed (Jan 15, 2026):**
- ✅ `DELETE /api/v1/games/:game_id/players/:player_id` - Kick player (creator only)
- ✅ `GET /api/v1/games/:game_id/participants` - List game participants with roles

### Key Features

#### 🎮 Turn-Based Gameplay
- **Ordered Turns**: Players take turns in sequence based on join order
- **Turn Validation**: Actions restricted to the current player's turn
- **Auto-Advance**: Turns automatically advance when player stands, busts, or finishes
- **Smart Skipping**: Turn system skips inactive players (standing/busted)

#### 👥 User Management
- **User Registration**: Create persistent user accounts with email/password
- **User Authentication**: Secure login system with JWT tokens
- **User Profiles**: User IDs linked to game sessions
- **Creator Tracking**: Games track which user created them

#### 📨 Game Invitations
- **Invitation System**: Users can invite others to join games
- **Timeout Control**: Configurable invitation expiration (default: 5 minutes, max: 1 hour)
- **Status Tracking**: Pending, Accepted, Declined, Expired states
- **Auto-Cleanup**: Expired invitations automatically detected

### Data Structures

#### User Model (Updated in Milestone 8)
```rust
pub struct User {
    pub id: Uuid,                        // Unique user identifier
    pub email: String,                   // Email address (unique)
    pub password_hash: String,           // Argon2id hashed password (M8)
    pub is_active: bool,                 // Account status (M8)
    pub last_login: Option<String>,      // Last login timestamp (M8)
    pub created_at: Option<String>,      // Account creation timestamp
    pub stats: Option<UserStats>,        // Player statistics
}
```

#### Game Invitation
```rust
pub struct GameInvitation {
    pub id: Uuid,
    pub game_id: Uuid,
    pub from_user_id: Uuid,    // Who sent the invitation
    pub to_user_id: Uuid,      // Who receives it
    pub status: InvitationStatus,
    pub timeout_seconds: u64,   // Configurable timeout
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

pub enum InvitationStatus {
    Pending,
    Accepted,
    Declined,
    Expired,
}
```

#### Player State
```rust
pub enum PlayerState {
    Active,      // Currently playing
    Standing,    // Decided to stop drawing
    Busted,      // Exceeded 21 points
}
```

#### Access Control System (Milestone 8)

**Game Roles:**
```rust
pub enum GameRole {
    Creator,    // User who created the game (all permissions)
    Player,     // Regular enrolled player (own actions only)
    Spectator,  // Future: read-only access
}
```

**Game Permissions:**
- `InvitePlayers` - Invite other users to join (Creator only)
- `KickPlayers` - Remove players from game (Creator only)
- `CloseEnrollment` - Manually close enrollment (Creator only)
- `FinishGame` - Manually finish game (Creator only)
- `ModifySettings` - Change game settings (Creator only)

**Game Participant:**
```rust
pub struct GameParticipant {
    pub user_id: Uuid,
    pub email: String,
    pub role: GameRole,
    pub joined_at: String,
}
```

**Access Control Methods:**
```rust
// Check if user can perform an action
game.can_user_perform(user_id, permission) -> bool

// Get user's role in the game
game.get_participant_role(user_id) -> Option<GameRole>

// Check if user is the creator
game.is_creator(user_id) -> bool

// Check if user is a participant
game.is_participant(user_id) -> bool
```

#### Enhanced Game Model
The `Game` struct now includes:
- `turn_order: Vec<String>` - Ordered list of player emails
- `current_turn_index: usize` - Index of current player's turn
- `creator_id: Uuid` - User who created the game
- Player states tracked via `PlayerState` enum

### New Services

#### UserService (Enhanced in Milestone 8)
```rust
// Register new user with password validation and Argon2id hashing
user_service.register(email, password) -> Result<Uuid>

// Login with constant-time password verification
user_service.login(email, password) -> Result<User>

// Change password with validation
user_service.change_password(user_id, old_password, new_password) -> Result<()>

// Get user by ID
user_service.get_user(user_id) -> Result<User>
```

#### InvitationService
```rust
// Create invitation with timeout (seconds)
invitation_service.create(from_user_id, to_user_id, game_id, timeout_seconds)

// Accept invitation
invitation_service.accept(invitation_id)

// Decline invitation
invitation_service.decline(invitation_id)

// Get pending invitations for user
invitation_service.get_pending_for_user(user_id)

// Cleanup expired invitations
invitation_service.cleanup_expired()
```

#### Enhanced GameService
```rust
// Create game now requires creator_id
game_service.create_game(emails, creator_id) -> Result<Uuid>

// Stand - player stops drawing cards
game_service.stand(game_id, email) -> Result<()>

// Turn management (automatic)
game.advance_turn()           // Move to next active player
game.get_current_player()     // Get email of current player
game.can_player_act(email)    // Check if player can act now
game.check_auto_finish()      // Auto-finish if all done
```

### Configuration

Add to `crates/blackjack-api/config.toml`:

```toml
[invitations]
default_timeout_seconds = 300    # 5 minutes default
max_timeout_seconds = 3600       # 1 hour maximum
```

**Environment Variables:**
```bash
export BLACKJACK_INVITATIONS_DEFAULT_TIMEOUT_SECONDS=300
export BLACKJACK_INVITATIONS_MAX_TIMEOUT_SECONDS=3600
```

### Updated JWT Structure

The JWT token claims now include user authentication:

```rust
pub struct Claims {
    pub sub: String,           // Subject (email)
    pub exp: usize,            // Expiration timestamp
    pub user_id: Uuid,         // NEW: User ID
    pub game_id: Option<Uuid>, // Optional: Game ID (backward compatible)
}
```

### Backward Compatibility

✅ **Fully Backward Compatible**:
- Existing endpoints continue to work unchanged
- `game_id` in JWT claims is now `Optional<Uuid>`
- Login endpoint accepts both old format (email + game_id) and new format (email + password for user auth)
- Rate limiting updated to use `user_id` instead of game-specific limits

### Implementation Status

#### ✅ Completed
- [x] Core data structures (User, GameInvitation, PlayerState)
- [x] Game turn management logic
- [x] UserService implementation (registration, login, get user)
- [x] InvitationService implementation (create, accept, decline, cleanup)
- [x] GameService updates for turn-based gameplay
- [x] JWT Claims structure with user_id
- [x] Configuration with invitation timeouts
- [x] Error handling for new error types
- [x] Middleware updates (rate limiting with user_id)
- [x] Full workspace compilation

#### ⏸️ Pending
- [ ] REST API handlers for user registration/login
- [ ] REST API handlers for invitations (create, list, accept, decline)
- [ ] REST API handlers for turn-based actions
- [ ] Comprehensive tests (25+ tests per PRD)
- [ ] Postman collection updates
- [ ] Full API documentation

### Technical Decisions

#### Password Security (✅ Implemented in Milestone 8)
- **Hashing Algorithm**: Argon2id (OWASP recommended)
- **Parameters**:
  - Memory cost: 19456 KiB (19 MiB)
  - Time cost: 2 iterations
  - Parallelism: 1 thread
  - Random 16-byte salt per hash
- **Security Features**:
  - Constant-time password verification (timing attack protection)
  - Email format validation (RFC 5322)
  - Password complexity validation
  - Account status tracking (`is_active` field)
  - Last login timestamp tracking

#### Turn Management
- Automatic turn advancement when player stands or busts
- Game auto-finishes when all players are standing/busted
- Turns skip inactive players automatically

#### Invitation Timeout
- Server-enforced maximum timeout (1 hour)
- Client can request shorter timeouts
- Expired invitations cleaned up on query

### Next Steps

See [docs/postman/ARCHITECTURE.md](docs/postman/ARCHITECTURE.md) for detailed architecture and implementation overview.

---

## Docker Deployment

### Building the Docker Image

```bash
# Build the image
docker build -t blackjack-api:latest .

# Run the container
docker run -p 8080:8080 \
  -e BLACKJACK_JWT_SECRET=your-production-secret \
  -e RUST_LOG=info \
  blackjack-api:latest
```

### Docker Compose (Optional)

Create a `docker-compose.yml`:

```yaml
version: '3.8'

services:
  api:
    build: .
    ports:
      - "8080:8080"
    environment:
      - BLACKJACK_JWT_SECRET=${BLACKJACK_JWT_SECRET}
      - BLACKJACK_SERVER_HOST=0.0.0.0
      - RUST_LOG=info
    restart: unless-stopped
```

Run with:

```bash
export BLACKJACK_JWT_SECRET=your-production-secret
docker-compose up -d
```

## CI/CD Pipeline

The project includes a GitHub Actions workflow (`.github/workflows/ci.yml`) that runs on every push and pull request:

### Pipeline Stages

1. **Test** - `cargo test --workspace`
2. **Lint** - `cargo clippy -- -D warnings`
3. **Format** - `cargo fmt --check`
4. **Build** - `cargo build --release`
5. **Docker Build** - Multi-stage Docker image build

### Running CI Locally

```bash
# Run all checks locally before pushing
cargo test --workspace
cargo clippy -- -D warnings
cargo fmt --check
cargo build --release
```

## Observability

### Structured Logging

The API uses `tracing` for structured, contextual logging:

```rust
// Logs include context from instrumentation
tracing::info!(
    game_id = %game_id,
    player_count = player_count,
    "Game created successfully"
);
```

### Log Levels

Control log verbosity with `RUST_LOG`:

- `error` - Only errors
- `warn` - Warnings and errors
- `info` - Normal operations (recommended for production)
- `debug` - Detailed debugging information
- `trace` - Very verbose, includes all details

**Example:**
```bash
# Production
RUST_LOG=info cargo run -p blackjack-api

# Development
RUST_LOG=debug cargo run -p blackjack-api

# Specific module debugging
RUST_LOG=blackjack_api::handlers=debug,blackjack_service=info cargo run -p blackjack-api
```

### Health Checks

Use health endpoints for monitoring:

```bash
# Basic liveness check
curl http://localhost:8080/health

# Readiness check (for Kubernetes probes)
curl http://localhost:8080/health/ready
```

## Testing

The project has comprehensive test coverage across all layers:

```bash
# Run all tests
cargo test --workspace

# Run tests with output
cargo test --workspace -- --nocapture

# Run tests for specific crate
cargo test -p blackjack-core
cargo test -p blackjack-service
cargo test -p blackjack-api

# Run specific test
cargo test -p blackjack-api test_config_defaults

# Run doc tests
cargo test --doc
```

### Test Coverage

- **Core (19 tests)**: Game logic, card deck, player mechanics, winner calculation
- **Service (12 tests)**: Game service, configuration, concurrency, error handling
- **API (13 tests)**: Config loading, error conversion, rate limiting, authentication
- **CLI (13 tests)**: Original CLI game tests
- **Doc tests (17)**: Documentation examples validation

**Total: 74 tests** covering all critical paths.

## Production Deployment Checklist

Before deploying to production:

- [ ] **Change JWT Secret**: Set strong `BLACKJACK_JWT_SECRET` via environment variable
- [ ] **Configure CORS**: Update `allowed_origins` in config.toml or via env var
- [ ] **Set Log Level**: Use `RUST_LOG=info` or `warn` for production
- [ ] **Enable HTTPS**: Use a reverse proxy (nginx, Caddy) with TLS termination
- [ ] **Set Rate Limits**: Adjust `requests_per_minute` based on your needs
- [ ] **Monitor Logs**: Integrate with log aggregation (ELK, Datadog, etc.)
- [ ] **Health Checks**: Configure orchestrator to use `/health` and `/health/ready`
- [ ] **Resource Limits**: Set appropriate CPU and memory limits in container orchestrator
- [ ] **Backup Strategy**: Plan for future database backups (when SQLite is integrated)

### Reverse Proxy Example (nginx)

```nginx
server {
    listen 80;
    server_name api.example.com;

    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

## Future Enhancements

The following features are planned but not yet implemented:

### WebSocket Real-Time Notifications

Blueprint is available in `crates/blackjack-api/src/websocket.rs`:

- Real-time game event broadcasting
- JWT authentication on connection
- Events: card drawn, Ace value changed, game finished

### SQLite Persistence

Migration files are prepared in `crates/blackjack-service/migrations/`:

- Persistent game state across restarts
- Player history and statistics
- Game replay capabilities

### Metrics and Monitoring

- Prometheus `/metrics` endpoint
- Counters: games created, cards drawn, rate limits hit
- Gauges: active games, total players
- Grafana dashboard templates

### Additional Features

- **Hot Configuration Reload**: Using `notify` crate to watch config.toml
- **Input Validation**: Using `validator` crate for request validation
- **Secrets Management**: Integration with HashiCorp Vault or AWS Secrets Manager
- **API Versioning**: Support for `/api/v2` alongside `/api/v1` with deprecation headers
- **Admin Endpoints**: Game management, player statistics, system metrics

See [`docs/PRD.md`](docs/PRD.md) for the complete product roadmap.

## Contributing

### Development Workflow

1. Create a feature branch
2. Make your changes
3. Run tests: `cargo test --workspace`
4. Run clippy: `cargo clippy -- -D warnings`
5. Format code: `cargo fmt`
6. Create a pull request

### Code Style

- Follow Rust naming conventions
- Add doc comments to public APIs
- Include examples in doc comments
- Write tests for new features
- Keep functions focused and small

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Authors

Developed as a learning project for building production-ready REST APIs in Rust.

## Acknowledgments

- Built with [Axum](https://github.com/tokio-rs/axum) - Ergonomic web framework
- Uses [Tokio](https://tokio.rs) - Async runtime for Rust
- Logging with [Tracing](https://github.com/tokio-rs/tracing) - Application-level tracing
- JWT with [jsonwebtoken](https://github.com/Keats/jsonwebtoken) - JSON Web Token implementation

### Running the CLI Game

**Prerequisites:**
- Rust 1.75 or higher
- Cargo package manager

**Build and run:**
```bash
# Build the project
cargo build

# Run the game
cargo run

# Run tests
cargo test
```

**Gameplay Example:**
```
Welcome to the Card Game!
Enter the number of players: 2

Player 1's turn:
Do you want a card? [Y/n]: y
You got the card: 5
Your current points: 5
Do you want a card? [Y/n]: y
You got the card: A
You have an Ace! Do you want it to count as 11 points instead of 1? [Y/n]: y
Your current points: 16
Do you want a card? [Y/n]: n
Player 1 finished with 16 points.

Player 2's turn:
...

==========================
Game Results:
==========================
Player 1 wins with 16 points!
Finished.
```

### Dependencies
- **rand 0.9.2**: Random card selection

## Future Development: Backend API System

This project is being transformed into a production-ready REST API backend system. See the complete [Product Requirements Document (PRD)](docs/PRD.md) for detailed information about the planned evolution.

### Planned Architecture
The system will be restructured into a Cargo workspace with multiple crates:

```
rust_blackjack/
├── crates/
│   ├── blackjack-core/      # Core game logic and domain models
│   ├── blackjack-service/   # Business logic and state management
│   ├── blackjack-api/       # REST API and HTTP handlers
│   └── blackjack-cli/       # Original CLI version (preserved)
├── docs/
│   └── PRD.md              # Detailed implementation plan
└── README.md
```

### Key Features (Planned)
- ✅ **REST API**: Versioned endpoints under `/api/v1`
- ✅ **JWT Authentication**: Secure player identification
- ✅ **Multi-player Games**: Shared game state for 1-10 players per game ID
- ✅ **52-Card Deck**: Realistic card deck with 4 suits
- ✅ **Card History**: Players can view all cards they've drawn
- ✅ **Flexible Ace Values**: Change Ace values multiple times during gameplay
- ✅ **Rate Limiting**: Prevent API abuse (configurable req/min)
- ✅ **Health Checks**: `/health` and `/health/ready` endpoints
- ✅ **Structured Logging**: Tracing with contextual information
- ✅ **External Configuration**: TOML config + environment variables
- ✅ **CI/CD Pipeline**: Automated testing, linting, and Docker builds
- ✅ **Production Ready**: Docker support, CORS, error handling

### Development Roadmap

See [PRD.md](docs/PRD.md) for the complete implementation plan:

**Completed Milestones:**
1. ✅ **Milestone 1**: Workspace Configuration and CI/CD
2. ✅ **Milestone 2**: Core Crate (game logic)
3. ✅ **Milestone 3**: Service Crate (state management)
4. ✅ **Milestone 4**: API Crate (authentication & config)
5. ✅ **Milestone 5**: REST Endpoints & Health Checks
6. ✅ **Milestone 6**: Tests, Documentation & Docker

**Completed Milestones (continued):**
7. ✅ **Milestone 7**: Turn-Based Gameplay and User Management (COMPLETE - Jan 14, 2026)
   - ✅ Phase 1: Game Enrollment Endpoints
   - ✅ Phase 2A: Game Invitation Endpoints
   - ✅ Phase 2B: Stand Endpoint
   - ✅ Phase 3: PlayerState & Turn Management
   - ✅ Phase 4: Additional Tests
8. ✅ **Milestone 8**: Security Hardening and Enhanced User Management (COMPLETE - Jan 15, 2026)
   - ✅ Argon2id Password Hashing
   - ✅ Password Complexity Validation
   - ✅ Role-Based Access Control (RBAC)
   - ✅ Security Headers (CSP, HSTS, X-Frame-Options)
   - ✅ Account Management (activation, deactivation, login tracking)
   - ✅ Password Change Endpoint
   - ✅ Kick Player Endpoint
   - ✅ Get Participants Endpoint

**Planned:**
9. ⏳ **Milestone 9**: SQLite Persistence and Database Layer

**Status**: 167/167 tests passing | All features complete through M8 | Production-ready API

## Contributing

This project is currently under active development. Contributions are welcome once the backend architecture is established.

## License

This project is licensed under the terms specified in the [LICENSE](LICENSE) file.

## Documentation

- **[Product Requirements Document (PRD)](docs/PRD.md)**: Complete technical specification for the backend transformation
- **Current Implementation**: See `src/main.rs` for the CLI version source code

## Version History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2025-12-23 | Initial CLI implementation with 1-10 player support |
| 1.0.0 | TBD | Backend REST API system (see PRD) |

---

**Current Status**: CLI Version Operational | Backend Development Planned  
**Next Steps**: See [Phase 1 in PRD](docs/PRD.md#phase-1-workspace-configuration-and-cicd)
