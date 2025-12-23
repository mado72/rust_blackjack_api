# Blackjack Card Game

A multi-player Blackjack card game implementation in Rust, evolving from a CLI application to a production-ready REST API backend system.

## Overview

This project implements a simplified Blackjack card game where 1-10 players compete to get the highest score without exceeding 21 points. The game features:

- **Multi-player support**: 1 to 10 players per game
- **Classic card values**: Numbered cards (2-10), Face cards (J, Q, K = 10), and Aces (1 or 11)
- **Strategic gameplay**: Players choose when to draw cards and can toggle Ace values
- **Winner determination**: Highest score ≤21 wins; ties are supported

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

## Current Implementation (CLI Version)

### Project Structure
```
rust_blackjack/
├── Cargo.toml
├── src/
│   └── main.rs          # CLI game implementation
├── docs/
│   └── PRD.md          # Product Requirements Document
├── LICENSE
└── README.md           # This file
```

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

See [PRD.md](docs/PRD.md) for the complete 6-phase implementation plan:

1. **Phase 1**: Workspace Configuration and CI/CD
2. **Phase 2**: Core Crate (game logic)
3. **Phase 3**: Service Crate (state management)
4. **Phase 4**: API Crate (authentication & config)
5. **Phase 5**: REST Endpoints & Health Checks
6. **Phase 6**: Tests, Documentation & Docker

**Status**: Currently in Planning Phase (v1.0.0)

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
