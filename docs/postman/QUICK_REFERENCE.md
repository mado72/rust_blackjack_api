# Blackjack API - Quick Reference Guide

## 🎮 Turn-Based Multiplayer Blackjack

Complete turn-based gameplay with user management and invitation system. All 16 endpoints are ready and tested!

**Core Features:**
- 🎮 Turn-based card drawing
- 👥 User registration and authentication  
- 📨 Invitation system with configurable timeouts
- 🔄 Automatic game completion
- 🏆 Real-time game state tracking

---

## 🚀 Quick Start

### 1. Start Server
```bash
cargo run -p blackjack-api
# Server: http://localhost:8080
```

### 2. Import to Postman
- Import: `Blackjack_API.postman_collection.json`
- Import: `Blackjack_API_Local.postman_environment.json`
- Select environment: **Blackjack API - Local**

### 3. Test Flow
```
Health Check → Register → Login → Create Game → Invite Players → 
Accept Invitation → Draw Cards → Stand → Results
```

---

## 📋 Available Endpoints (16 total)

**Status:** ✅ All implemented and functional

| Endpoint | Method | Auth? | Description |
|----------|--------|-------|-------------|
| `/health` | GET | ❌ | Server health status |
| `/health/ready` | GET | ❌ | Component readiness |
| `/api/v1/auth/register` | POST | ❌ | Register new user |
| `/api/v1/auth/login` | POST | ❌ | User authentication |
| `/api/v1/games` | POST | ✅ | Create new game |
| `/api/v1/games/:id` | GET | ✅ | Get game state with turns |
| `/api/v1/games/:id/draw` | POST | ✅ | Draw card (turn validated) |
| `/api/v1/games/:id/ace` | PUT | ✅ | Change Ace value |
| `/api/v1/games/:id/stand` | POST | ✅ | Player stands |
| `/api/v1/games/:id/finish` | POST | ✅ | Finish game manually |
| `/api/v1/games/:id/results` | GET | ✅ | View game results |
| `/api/v1/games/:id/invitations` | POST | ✅ | Create invitation |
| `/api/v1/invitations/pending` | GET | ✅ | List pending invitations |
| `/api/v1/invitations/:id/accept` | POST | ✅ | Accept invitation |
| `/api/v1/invitations/:id/decline` | POST | ✅ | Decline invitation |

---

## 🔐 Autenticação

### Token JWT
- Obtido via: `POST /api/v1/auth/login`
- Válido por: **24 horas**
- Header: `Authorization: Bearer <token>`
- Automaticamente gerenciado no Postman ✅

---

## 📝 Variáveis Principais

| Variável | Auto? | Descrição |
|----------|-------|-----------|
| `base_url` | ❌ | `http://localhost:8080` |
| `game_id` | ✅ | UUID do jogo (salvo no Create Game) |
| `jwt_token` | ✅ | Token JWT (salvo no Login) |
| `player_email` | ❌ | Email do jogador atual |
| `card_id` | ✅ | UUID de carta Ás (salvo no Draw Card) |

---

## 🎮 Exemplos de Requests

### Criar Jogo
```json
POST /api/v1/games
{
  "emails": [
    "player1@example.com",
    "player2@example.com"
  ]
}
```

### Login
```json
POST /api/v1/auth/login
{
  "email": "player1@example.com",
  "game_id": "{{game_id}}"
}
```

### Logout
```
POST /api/v1/auth/logout
Authorization: Bearer {{jwt_token}}
```

### Comprar Carta
```
POST /api/v1/games/{{game_id}}/draw
Authorization: Bearer {{jwt_token}}
```

### Mudar Ás
```json
PUT /api/v1/games/{{game_id}}/ace
Authorization: Bearer {{jwt_token}}
{
  "card_id": "{{card_id}}",
  "as_eleven": false
}
```

---

## 📊 Status Codes

| Code | Meaning | When It Occurs |
|--------|-------------|---------------|
| 200 | OK | Successful request |
| 400 | Bad Request | Invalid data (UUID, player count) |
| 401 | Unauthorized | Missing or invalid token |
| 403 | Forbidden | Player not in game / game finished |
| 404 | Not Found | Game/player/card not found |
| 409 | Conflict | Game already finished / game not finished |
| 410 | Gone | Empty deck |
| 429 | Too Many Requests | Rate limit exceeded |
| 500 | Internal Server Error | Server error |

---

## ⚠️ Common Errors

### 401 Unauthorized
**Cause**: Invalid or expired JWT token  
**Solution**: Login again

### 403 Forbidden - Player not in game
**Cause**: Email not in player list  
**Solution**: Use email that was included in Create Game

### 403 Forbidden - Game finished
**Cause**: Trying to play after finishing  
**Solution**: Create a new game

### 404 Not Found - Game not found
**Cause**: Invalid or non-existent `game_id`  
**Solution**: Verify the UUID or create new game

### 409 Conflict - Game not finished
**Cause**: Trying to see results before finishing  
**Solution**: Call `POST /api/v1/games/:id/finish` first

---

## 🧪 Testing Tools

### Postman
```
✅ Best for: Visual interface, debugging
📁 File: Blackjack_API.postman_collection.json
📖 Guide: POSTMAN_GUIDE.md
```

### VS Code REST Client
```
✅ Best for: Quick tests in editor
📁 File: api_tests.http
💡 Extension: humao.rest-client
```

### PowerShell Script
```
✅ Best for: Complete automated tests
📁 File: test_api.ps1
▶️ Run: .\test_api.ps1
```

### cURL
```
✅ Best for: Command line, scripts
📁 File: CURL_EXAMPLES.md
🐧 Linux/Mac ready
```

---

## 🎯 Test Scenarios

### Basic Test (1 player)
1. Create Game with 1 email
2. Login
3. Draw 2-3 cards
4. Finish Game
5. Get Results

### Multi-player Test
1. Create Game with 3+ emails
2. Login as player 1
3. Draw cards for player 1
4. Switch token (login as player 2)
5. Draw cards for player 2
6. Finish Game
7. Get Results

### Ace Test
1. Create Game
2. Login
3. Draw until getting an Ace (script saves ID automatically)
4. Set Ace Value to 11
5. Set Ace Value to 1
6. See point difference

### Bust Test
1. Create Game
2. Login
3. Draw multiple cards until busting (> 21)
4. Verify `busted: true`
5. Finish and verify loss

---

## 🔄 Recommended Workflow

### Development
```bash
# Terminal 1: Server
cargo run -p blackjack-api

# Terminal 2: Tests
cargo test --workspace

# Terminal 3: API Tests
.\test_api.ps1
```

### Debugging
1. Use Postman for individual requests
2. Check logs in server terminal
3. Use `RUST_LOG=debug` for detailed logs

### CI/CD
```bash
# Complete tests
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --workspace --check

# Production build
cargo build --release -p blackjack-api
```

---

## 📚 Complete Documentation

- **API Endpoints**: [../../crates/blackjack-api/src/handlers.rs](../../crates/blackjack-api/src/handlers.rs)
- **Postman**: [POSTMAN_GUIDE.md](POSTMAN_GUIDE.md)
- **cURL**: [CURL_EXAMPLES.md](CURL_EXAMPLES.md)
- **PRD**: [../PRD.md](../PRD.md)

---

## 🆘 Troubleshooting

### Server won't start
```bash
# Check if port is in use
netstat -ano | findstr :8080

# Change port
$env:BLACKJACK_SERVER_PORT=3000
cargo run -p blackjack-api
```

### Variables not saving in Postman
1. Check selected environment (top right corner)
2. Verify using `{{variable}}` syntax correctly
3. Execute requests in correct order

### Token expires quickly
```toml
# Adjust in config.toml
[jwt]
expiration_hours = 48  # 2 days
```

### Rate limit too restrictive
```toml
# Adjust in config.toml
[rate_limit]
requests_per_minute = 30  # Increase
```

---

## ⚡ Useful Shortcuts

### Postman
- `Ctrl+Enter`: Send request
- `Ctrl+E`: Open environments
- `Ctrl+Shift+C`: Open console

### VS Code REST Client
- `Ctrl+Alt+R`: Send request
- `Ctrl+Alt+C`: Cancel request
- `Ctrl+Alt+H`: View history

---

**Version**: 1.0.0  
**Last updated**: January 2026
