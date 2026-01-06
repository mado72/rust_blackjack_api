-- Future SQLite schema for persistence
-- This migration is commented out and will be implemented in a future phase
-- when SQLx dependency is activated

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
