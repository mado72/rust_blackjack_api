# Product Requirements Document - Blackjack Multi-Player Backend System

**Version:** 1.0.0  
**Last Updated:** December 23, 2025  
**Status:** Planning Phase

## Document Overview

This document details the transformation of the CLI blackjack game into a production-ready REST backend system with versioned API, JWT authentication, multi-player game management (1-10 players per game), shared 52-card deck, ordered card history, flexible Ace value changes, rate limiting, structured logging, health checks, standardized errors, external configuration, and CI/CD pipeline.

---

## Phase 1: Workspace Configuration and CI/CD

**Status:** `completed`  
**Dependencies:** None  
**Estimated Effort:** 4 hours

### Tasks

- [x] Create workspace root `Cargo.toml` with members: `["crates/blackjack-core", "crates/blackjack-service", "crates/blackjack-api", "crates/blackjack-cli"]`
- [x] Create `crates/blackjack-core/Cargo.toml` with dependencies: rand 0.9.2, uuid v4, serde derive
- [x] Create `crates/blackjack-service/Cargo.toml` with dependencies: thiserror 2, tracing 0.1
- [x] Create `crates/blackjack-api/Cargo.toml` with dependencies: axum 0.7, tokio full, serde derive, serde_json, jsonwebtoken 9, tower-http cors 0.6, tower 0.5, tracing 0.1, tracing-subscriber 0.3 env-filter, config 0.14, dotenv 0.15
- [x] Add commented future dependencies: `# Future: sqlx, metrics, metrics-exporter-prometheus, notify, validator`
- [x] Move `src/main.rs` to `crates/blackjack-cli/src/main.rs` (preserve original CLI version)
- [x] Create `.github/workflows/ci.yml` with jobs:
  - [x] `test`: cargo test --workspace
  - [x] `lint`: cargo clippy -- -D warnings
  - [x] `format`: cargo fmt --check
  - [x] `build`: cargo build --release
  - [x] `docker-build`: multi-stage Dockerfile

### Acceptance Criteria

- ✅ Workspace builds successfully with `cargo build --workspace`
- ✅ CI pipeline runs all checks on push/PR
- ✅ Original CLI version preserved and functional

---

## Phase 2: Core Crate (blackjack-core)

**Status:** `completed`  
**Dependencies:** Phase 1  
**Estimated Effort:** 8 hours

### Tasks

- [x] Expand CARDS constant from 13 to 52 cards (4 copies of each type)
- [x] Add suits: ["Hearts", "Diamonds", "Clubs", "Spades"]
- [x] Create `Card` struct with `#[derive(Debug, Clone, Serialize, Deserialize)]`:
  - [x] Fields: `id: Uuid, name: String, value: u8, suit: String`
- [x] Create `Player` struct:
  - [x] Fields: `email: String, points: u8, cards_history: Vec<Card>, ace_values: HashMap<Uuid, bool>, busted: bool`
  - [x] `ace_values` maps card_id to is_eleven (true = 11 points, false = 1 point)
- [x] Create `PlayerSummary` struct:
  - [x] Fields: `points: u8, cards_count: usize, busted: bool`
- [x] Create `GameResult` struct:
  - [x] Fields: `winner: Option<String>, tied_players: Vec<String>, highest_score: u8, all_players: HashMap<String, PlayerSummary>`
- [x] Create `Game` struct:
  - [x] Fields: `id: Uuid, players: HashMap<String, Player>, available_cards: Vec<Card>, finished: bool`
- [x] Implement `Game::new(player_emails)` method:
  - [x] Validate 1-10 unique non-empty emails
  - [x] Initialize 52-card deck
  - [x] Add `#[tracing::instrument]` attribute
- [x] Implement `Game::draw_card(email) -> Result<Card, GameError>`:
  - [x] Remove random card from deck if `!finished`
  - [x] Update player's cards_history
  - [x] Add `#[tracing::instrument]` attribute
- [x] Implement `Game::set_ace_value(email, card_id, as_eleven)`:
  - [x] Recalculate player points if `!finished`
  - [x] Allow multiple changes to same Ace
  - [x] Add `#[tracing::instrument]` attribute
- [x] Implement `Game::finish_game()`:
  - [x] Set `finished = true`
- [x] Implement `Game::calculate_results() -> GameResult`:
  - [x] Based on `determine_winner` logic from main.rs lines 138-167
  - [x] Handle single winner, ties, all-bust scenarios
- [x] Document all public structs, methods and functions:
  - [x] Add comprehensive doc comments with examples
  - [x] Document struct fields and their purposes
  - [x] Include usage examples for key methods
  - [x] Add inline comments for complex logic

### Acceptance Criteria

- ✅ All structs serialize/deserialize correctly to JSON
- ✅ Deck contains exactly 52 unique cards (4 of each type)
- ✅ Ace value can be changed multiple times
- ✅ Game finished prevents further operations
- ✅ All methods have tracing instrumentation

---

## Phase 3: Service Crate with Migrations, Logging and Config

**Status:** `completed`  
**Dependencies:** Phase 2  
**Estimated Effort:** 6 hours

### Tasks

- [x] Create `ServiceConfig` struct:
  - [x] Fields: `max_players: u8, min_players: u8`
  - [x] Load from env vars with defaults (1-10)
- [x] Create `GameService` struct:
  - [x] Fields: `games: Arc<Mutex<HashMap<Uuid, Game>>>, config: ServiceConfig`
- [x] Create `crates/blackjack-service/migrations/` directory
- [x] Create `20250101000000_initial_schema.sql` with commented SQL:
  ```sql
  -- CREATE TABLE games (
  --   id TEXT PRIMARY KEY,
  --   finished BOOLEAN NOT NULL DEFAULT 0,
  --   created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
  -- );
  -- CREATE TABLE players (
  --   id TEXT PRIMARY KEY,
  --   email TEXT NOT NULL,
  --   game_id TEXT NOT NULL REFERENCES games(id),
  --   points INTEGER NOT NULL,
  --   busted BOOLEAN NOT NULL,
  --   UNIQUE(email, game_id)
  -- );
  -- CREATE TABLE cards_history (
  --   id TEXT PRIMARY KEY,
  --   player_id TEXT NOT NULL REFERENCES players(id),
  --   card_id TEXT NOT NULL,
  --   name TEXT NOT NULL,
  --   suit TEXT NOT NULL,
  --   value INTEGER NOT NULL,
  --   drawn_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
  -- );
  -- CREATE INDEX idx_players_game ON players(game_id);
  -- CREATE INDEX idx_cards_player ON cards_history(player_id);
  ```
- [x] Implement `GameService::create_game(emails) -> Result<Uuid, GameError>`:
  - [x] Validate against `config.min_players` and `config.max_players`
  - [x] Log game creation with `tracing::info!`
  - [x] Add `#[tracing::instrument(skip(self), fields(game_id))]`
- [x] Implement `GameService::draw_card(game_id, email) -> Result<DrawCardResponse>`:
  - [x] Return `{card: Card, current_points: u8, busted: bool, cards_remaining: usize, cards_history: Vec<Card>}`
  - [x] Log card drawn with `tracing::debug!`
  - [x] Add instrumentation with game_id and player_email fields
- [x] Implement `GameService::set_ace_value(game_id, email, card_id, as_eleven) -> Result<PlayerStateResponse>`:
  - [x] Return `{points: u8, busted: bool}`
  - [x] Add instrumentation
- [x] Implement `GameService::get_game_state(game_id) -> Result<GameStateResponse>`:
  - [x] Return `{players: HashMap<email, PlayerInfo>, cards_in_deck: usize, finished: bool}`
  - [x] `PlayerInfo {points: u8, cards_history: Vec<Card>, busted: bool}`
- [x] Implement `GameService::finish_game(game_id) -> Result<GameResult>`:
  - [x] Log winner with `tracing::info!`
- [x] Create `GameError` enum with thiserror derives:
  - [x] Variants: GameNotFound, PlayerNotInGame, PlayerAlreadyBusted, InvalidPlayerCount, InvalidEmail, DeckEmpty, GameAlreadyFinished
- [x] Document all service layer code:
  - [x] Add module-level documentation explaining service architecture
  - [x] Document ServiceConfig and GameService structs
  - [x] Add doc comments to all public methods with examples
  - [x] Document error types and when they occur
  - [x] Include inline comments for concurrency patterns

### Acceptance Criteria

- ✅ Service handles concurrent access safely with Arc/Mutex
- ✅ All operations properly instrumented with tracing
- ✅ Configuration loaded from environment variables
- ✅ SQL migrations documented for future implementation
- ✅ Comprehensive error types with descriptive messages

---

## Phase 4: API Crate - External Configuration and Authentication

**Status:** `completed`  
**Dependencies:** Phase 3  
**Estimated Effort:** 8 hours

### Tasks

- [x] Create `crates/blackjack-api/config.toml`:
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
- [x] Create `AppConfig` struct using `config` crate:
  - [x] Load from `config.toml`
  - [x] Override with env vars `BLACKJACK_*`
- [x] Create `.env.example` with variables:
  - [x] `BLACKJACK_JWT_SECRET`
  - [x] `BLACKJACK_SERVER_PORT`
  - [x] `RUST_LOG`
- [x] Initialize tracing in main:
  - [x] `tracing_subscriber::fmt().with_env_filter().init()`
- [x] Create `Claims` struct:
  - [x] Fields: `email: String, game_id: String, exp: usize`
- [x] Create `ApiError` struct:
  - [x] Fields: `message: String, code: String, status: u16, details: Option<HashMap<String, String>>`
  - [x] Implement `IntoResponse` trait
- [x] Create `RateLimiter` struct:
  - [x] Use `config.rate_limit.requests_per_minute`
  - [x] Track requests per `{game_id}:{email}` key
  - [x] Use `Arc<Mutex<HashMap<String, VecDeque<Instant>>>>`
- [x] Implement `POST /api/v1/auth/login`:
  - [x] Accept `{email: String, game_id: String}`
  - [x] Validate via `service.get_game_state()`
  - [x] Generate JWT with `config.jwt.expiration_hours` and `config.jwt.secret`
  - [x] Return `{token: String, expires_in: usize}`
  - [x] Log authentication attempt with game_id and email
- [x] Create `auth_middleware`:
  - [x] Extract Bearer token from Authorization header
  - [x] Decode JWT using `jsonwebtoken::decode`
  - [x] Inject `Claims` via Axum Extension
  - [x] Return `ApiError {status: 401, code: "UNAUTHORIZED"}` on failure
  - [x] Log authentication with `tracing::debug!`
- [x] Create `rate_limit_middleware`:
  - [x] Check request limit per player
  - [x] Return `ApiError {status: 429, code: "RATE_LIMIT_EXCEEDED"}` if exceeded
  - [x] Log excess with `tracing::warn!`
- [x] Create `version_deprecation_middleware`:
  - [x] Add headers `X-API-Deprecated`, `X-API-Sunset-Date`
  - [x] Calculate sunset date from `config.api.version_deprecation_months`
- [x] Document all API infrastructure code:
  - [x] Add comprehensive module documentation (config.rs, error.rs, auth.rs, rate_limiter.rs, middleware.rs, handlers.rs, lib.rs, main.rs)
  - [x] Document configuration loading and environment variable precedence
  - [x] Add examples for middleware usage and chaining
  - [x] Document error handling patterns and ApiError structure
  - [x] Include authentication flow documentation
  - [x] Document rate limiting algorithm and implementation
  - [x] Add startup sequence documentation in main.rs

### Acceptance Criteria

- ✅ Configuration loads from file and env vars (env vars take precedence)
- ✅ JWT authentication works with configurable secret and expiration
- ✅ Rate limiting enforces configured limits per player
- ✅ All errors return standardized JSON format
- ✅ Middleware properly chains and injects context
- ✅ All modules fully documented with examples
- ✅ 13 integration tests passing (config, state, errors, rate limiter, service conversion)

---

## Phase 5: API Crate - REST Endpoints, Health Checks and WebSocket Blueprint

**Status:** `pending`  
**Dependencies:** Phase 4  
**Estimated Effort:** 10 hours

### Tasks

- [ ] Create `crates/blackjack-api/src/websocket.rs` with commented blueprint:
  ```rust
  /* TODO: WebSocket real-time notifications
   * 
   * Authentication: First message after handshake must be:
   * {"type": "auth", "token": "JWT_TOKEN_HERE"}
   * 
   * Struct GameNotification {
   *   event_type: String,  // "draw_card", "ace_changed", "game_finished"
   *   player_email: String,
   *   game_id: String,
   *   data: serde_json::Value
   * }
   * 
   * async fn websocket_handler(
   *   ws: WebSocketUpgrade,
   *   State(service): State<Arc<GameService>>
   * ) -> Response {
   *   ws.on_upgrade(|socket| handle_socket(socket, service))
   * }
   * 
   * async fn handle_socket(socket: WebSocket, service: Arc<GameService>) {
   *   // 1. Wait for auth message with JWT
   *   // 2. Validate token and extract game_id
   *   // 3. Subscribe to game notifications
   *   // 4. Send notifications on game events
   * }
   */
  ```
- [ ] Implement `GET /health`:
  - [ ] Return `{status: "healthy", uptime_seconds: u64, version: "1.0.0"}`
- [ ] Implement `GET /health/ready`:
  - [ ] Return `{ready: true, checks: {memory: "ok", config: "loaded", future_sqlite: "pending", future_metrics: "pending"}}`
- [ ] Implement `POST /api/v1/games`:
  - [ ] Accept `{emails: Vec<String>}`
  - [ ] Validate array of 1-10 unique emails
  - [ ] Return `{game_id: Uuid, message: String, player_count: usize}`
  - [ ] On error: `ApiError {code: "INVALID_PLAYER_COUNT", details: {"min": "1", "max": "10", "provided": "X"}}`
  - [ ] Log game creation
- [ ] Implement `GET /api/v1/games/:game_id` (protected):
  - [ ] Return `GameStateResponse`
  - [ ] Include complete cards_history for all players
- [ ] Implement `POST /api/v1/games/:game_id/draw` (protected):
  - [ ] Extract email from JWT Claims
  - [ ] Call `service.draw_card()`
  - [ ] Return `DrawCardResponse`
  - [ ] On finished game: `ApiError {code: "GAME_FINISHED"}`
  - [ ] Log card draw
- [ ] Implement `PUT /api/v1/games/:game_id/ace` (protected):
  - [ ] Accept `{card_id: Uuid, as_eleven: bool}`
  - [ ] Return `PlayerStateResponse`
  - [ ] Allow multiple changes to same Ace
- [ ] Implement `POST /api/v1/games/:game_id/finish` (protected):
  - [ ] Call `service.finish_game()`
  - [ ] Return `GameResult`
  - [ ] Log game finalization with winner
- [ ] Implement `GET /api/v1/games/:game_id/results` (protected):
  - [ ] Return `GameResult`
  - [ ] Error if game not finished
- [ ] Configure CORS:
  - [ ] Use `config.cors.allowed_origins`
  - [ ] Apply via tower-http middleware
- [ ] Configure server:
  - [ ] Bind to `config.server.host:config.server.port`
  - [ ] Default port 8080
- [ ] Document all REST endpoints and handlers:
  - [ ] Add module documentation for handlers module
  - [ ] Document each endpoint handler with request/response examples
  - [ ] Include authentication requirements in doc comments
  - [ ] Document error scenarios for each endpoint
  - [ ] Add examples of successful and failed requests
  - [ ] Document WebSocket blueprint thoroughly
  - [ ] Include CORS configuration documentation

### Acceptance Criteria

- All endpoints versioned under `/api/v1`
- Health checks return proper status
- Protected endpoints require valid JWT
- Rate limiting applied to all protected endpoints
- CORS configured for allowed origins
- WebSocket blueprint documented for future implementation
- All operations logged with structured tracing

---

## Phase 6: Tests, Documentation and Docker

**Status:** `pending`  
**Dependencies:** Phase 5  
**Estimated Effort:** 12 hours

### Tasks

#### Core Tests
- [ ] Create `crates/blackjack-core/tests/integration_tests.rs`:
  - [ ] Test deck has exactly 52 cards
  - [ ] Test 4 cards of each type (A, 2-10, J, Q, K)
  - [ ] Test deck exhaustion returns `GameError::DeckEmpty`
  - [ ] Test Ace value can be changed multiple times
  - [ ] Test game finished prevents draw/ace operations
  - [ ] Test JSON serialization/deserialization of all structs
  - [ ] Migrate tests from main.rs lines 194-347
  - [ ] Adapt `determine_winner` tests for `calculate_results()`

#### Service Tests
- [ ] Create `crates/blackjack-service/tests/service_tests.rs`:
  - [ ] Test concurrent access with Arc/Mutex (multiple threads drawing simultaneously)
  - [ ] Test race conditions on shared deck
  - [ ] Test config validation (min_players, max_players)
  - [ ] Test logging with mock tracing subscriber
  - [ ] Verify correct spans and fields in logs

#### API Tests
- [ ] Create `crates/blackjack-api/tests/api_tests.rs`:
  - [ ] Test rate limiting (11+ requests in 1 minute returns 429)
  - [ ] Test API versioning (`/api/v1` prefix)
  - [ ] Test deprecation headers presence
  - [ ] Test health checks return 200
  - [ ] Test config loading from env vars
  - [ ] Test `ApiError` format with details field
  - [ ] Test JWT authentication flow
  - [ ] Test CORS headers

#### Docker
- [ ] Create `Dockerfile` multi-stage:
  - [ ] Build stage: `cargo build --release`
  - [ ] Runtime stage: copy binary
  - [ ] `EXPOSE 8080`
  - [ ] `CMD ["blackjack-api"]`
- [ ] Create `.dockerignore`:
  - [ ] `target/`
  - [ ] `*.log`
  - [ ] `.env`

#### Code Documentation Review
- [ ] Review and validate all inline documentation:
  - [ ] Ensure all public APIs have doc comments
  - [ ] Verify all examples in doc comments are correct
  - [ ] Check that complex algorithms have inline explanations
  - [ ] Validate that error types are well documented
  - [ ] Ensure module-level docs explain architecture
- [ ] Generate and review `cargo doc` output:
  - [ ] Fix any broken links in documentation
  - [ ] Ensure examples compile and run
  - [ ] Verify documentation coverage is comprehensive

#### Documentation
- [ ] Create `README.md` with sections:
  - [ ] **Project Structure**: workspace layout
  - [ ] **Configuration**: document `config.toml`, env vars `BLACKJACK_*`, `.env.example`
  - [ ] **Development**:
    - [ ] Setup: `cargo build --workspace`
    - [ ] Run API: `cargo run -p blackjack-api`
    - [ ] Required env vars: `JWT_SECRET`, `RUST_LOG=debug`
  - [ ] **CI/CD**: GitHub Actions workflow (test, lint, format, build, docker)
  - [ ] **Observability**: structured logs with tracing, health checks
  - [ ] **Future Enhancements**:
    - [ ] SQLite migration with `sqlx migrate run`
    - [ ] v1/v2 simultaneous support
    - [ ] WebSockets for real-time notifications
    - [ ] Prometheus metrics endpoint `/metrics`
    - [ ] Hot reload config with `notify` crate
    - [ ] Config validation with `Validate` trait
    - [ ] Secrets management (HashiCorp Vault/AWS Secrets Manager)
  - [ ] **API Examples**: complete curl flow
    - [ ] Create game
    - [ ] Login all players
    - [ ] Draw cards
    - [ ] Change Ace values
    - [ ] View game state
    - [ ] Finish game
    - [ ] Get results
  - [ ] **API Reference**: all endpoints with request/response schemas
  - [ ] **Production Deployment**: Docker, reverse proxy, external config, log aggregation

### Acceptance Criteria

- All tests pass with `cargo test --workspace`
- Test coverage includes concurrent scenarios
- Docker image builds and runs successfully
- Documentation is comprehensive and clear
- Curl examples are tested and working
- CI/CD pipeline executes all phases

---

## Future Enhancements (Out of Scope for v1.0)

### Hot Reload Configuration
- Implement config file watcher using `notify` crate
- Reload rate limits and CORS origins without server restart
- Useful for production environment dynamic adjustments

### Configuration Validation
- Create `Validate` trait for `AppConfig`
- Validate ranges at startup:
  - Port: 1024-65535
  - Rate limit: > 0
  - JWT expiration: > 0
  - Max players >= min players
- Fail fast with clear error messages

### Secrets Management
- Integrate with HashiCorp Vault or AWS Secrets Manager
- Replace plaintext env vars for `JWT_SECRET`
- Automatic secret rotation for database credentials
- Production-grade security for sensitive configuration

### Metrics and Observability
- Add `metrics` and `metrics-exporter-prometheus` dependencies
- Expose `GET /metrics` endpoint for Prometheus
- Track counters: games created, cards drawn, rate limits hit
- Track gauges: active games, total players
- Integration with Grafana dashboards

### WebSocket Real-Time Notifications
- Implement blueprint from `websocket.rs`
- Authenticate via first message with JWT
- Broadcast game events to subscribed players
- Support multiple concurrent connections per game

### SQLite Persistence
- Uncomment `sqlx` dependency
- Run migrations from `migrations/` directory
- Replace in-memory HashMap with database storage
- Add database connection to health check

### API Versioning Evolution
- Implement `/api/v2` alongside `/api/v1`
- Maintain v1 for 6 months (configurable deprecation period)
- Return deprecation headers in v1 responses
- Document migration guide for clients

---

## Version History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2025-12-23 | Team | Initial PRD creation with 6 phases |

---
