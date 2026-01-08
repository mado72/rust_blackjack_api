# Blackjack Multi-Player Backend System

A production-ready REST API backend for multi-player Blackjack card games, built with Rust using Axum, with JWT authentication, rate limiting, structured logging, and comprehensive observability.

## Overview

This project provides a complete backend system for managing multi-player Blackjack games (1-10 players per game) with:

- **RESTful API**: Versioned endpoints under `/api/v1` with OpenAPI-style documentation
- **JWT Authentication**: Secure player authentication per game session
- **Rate Limiting**: Per-player request throttling using sliding window algorithm
- **Real-time Ready**: WebSocket blueprint for future real-time notifications
- **Observability**: Structured logging with tracing, health checks, and metrics-ready architecture
- **Production-Grade**: External configuration, CORS support, graceful error handling
- **Multi-player Support**: 1-10 players per game with independent state management
- **Flexible Gameplay**: Dynamic Ace values (1 or 11), ordered card history, bust detection

## Game Rules

### Card Values
- **Numbered cards (2-9)**: Face value
- **10, Jack, Queen, King**: 10 points each
- **Ace**: 1 point (can be changed to 11 points at player's discretion)

### Gameplay Flow
1. Game starts with a configurable number of players (1-10)
2. Each player takes turns drawing cards from the deck
3. After drawing a card:
   - If it's an Ace, player chooses to count it as 1 or 11 points
   - Player sees their current total score
   - Player decides whether to draw another card or stop
4. Player's turn ends when:
   - They choose to stop drawing
   - They exceed 21 points (bust)
5. After all players finish, the winner is determined

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

#### POST /api/v1/auth/login

Authenticate a player for a game session. Returns a JWT token.

**Request:**
```json
{
  "email": "player1@example.com",
  "game_id": "550e8400-e29b-41d4-a716-446655440000"
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
- `400` - Invalid game ID format
- `403` - Player not in game
- `404` - Game not found

### Game Management

#### POST /api/v1/games

Create a new game with 1-10 players.

**Request:**
```json
{
  "emails": [
    "player1@example.com",
    "player2@example.com",
    "player3@example.com"
  ]
}
```

**Response (200 OK):**
```json
{
  "game_id": "550e8400-e29b-41d4-a716-446655440000",
  "message": "Game created successfully",
  "player_count": 3
}
```

**Errors:**
- `400` - Invalid player count (min: 1, max: 10)

#### GET /api/v1/games/:game_id

Get current game state. **Requires authentication.**

**Headers:**
```
Authorization: Bearer <jwt_token>
```

**Response (200 OK):**
```json
{
  "players": {
    "player1@example.com": {
      "points": 18,
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
    }
  },
  "cards_in_deck": 48,
  "finished": false
}
```

**Errors:**
- `401` - Unauthorized (missing or invalid token)
- `403` - Token is for a different game
- `404` - Game not found

#### POST /api/v1/games/:game_id/draw

Draw a card for the authenticated player. **Requires authentication.**

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
  "cards_remaining": 47,
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
  ]
}
```

**Errors:**
- `401` - Unauthorized
- `403` - Game already finished
- `404` - Game or player not found
- `410` - Deck is empty

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

See [PRD.md](docs/PRD.md) for the complete 6-milestone implementation plan:

1. **Milestone 1**: Workspace Configuration and CI/CD
2. **Milestone 2**: Core Crate (game logic)
3. **Milestone 3**: Service Crate (state management)
4. **Milestone 4**: API Crate (authentication & config)
5. **Milestone 5**: REST Endpoints & Health Checks
6. **Milestone 6**: Tests, Documentation & Docker

**Status**: Currently in Planning Milestone (v1.0.0)

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
