use blackjack_core::{Card, Game, GameError as CoreGameError, GameResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use uuid::Uuid;

/// Service-level errors with thiserror derives
#[derive(Error, Debug)]
pub enum GameError {
    #[error("Game not found")]
    GameNotFound,
    #[error("Player not in this game")]
    PlayerNotInGame,
    #[error("Player already busted")]
    PlayerAlreadyBusted,
    #[error("Invalid player count (must be between {min} and {max}, got {provided})")]
    InvalidPlayerCount { min: u8, max: u8, provided: usize },
    #[error("Invalid email: {0}")]
    InvalidEmail(String),
    #[error("No more cards in the deck")]
    DeckEmpty,
    #[error("Game has already finished")]
    GameAlreadyFinished,
    #[error("Core game error: {0}")]
    CoreError(#[from] CoreGameError),
}

/// Configuration for the game service
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub max_players: u8,
    pub min_players: u8,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            max_players: 10,
            min_players: 1,
        }
    }
}

impl ServiceConfig {
    /// Load configuration from environment variables with defaults
    pub fn from_env() -> Self {
        let max_players = std::env::var("BLACKJACK_MAX_PLAYERS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10);

        let min_players = std::env::var("BLACKJACK_MIN_PLAYERS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);

        Self {
            max_players,
            min_players,
        }
    }
}

/// Response for draw card operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawCardResponse {
    pub card: Card,
    pub current_points: u8,
    pub busted: bool,
    pub cards_remaining: usize,
    pub cards_history: Vec<Card>,
}

/// Response for player state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStateResponse {
    pub points: u8,
    pub busted: bool,
}

/// Information about a player in the game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub points: u8,
    pub cards_history: Vec<Card>,
    pub busted: bool,
}

/// Response for game state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStateResponse {
    pub players: HashMap<String, PlayerInfo>,
    pub cards_in_deck: usize,
    pub finished: bool,
}

/// Main game service managing multiple games
pub struct GameService {
    games: Arc<Mutex<HashMap<Uuid, Game>>>,
    config: ServiceConfig,
}

impl GameService {
    /// Creates a new game service with the given configuration
    pub fn new(config: ServiceConfig) -> Self {
        Self {
            games: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }

    /// Creates a new game service with default configuration
    pub fn new_default() -> Self {
        Self::new(ServiceConfig::default())
    }

    /// Creates a new game with the specified players
    #[tracing::instrument(skip(self), fields(game_id))]
    pub fn create_game(&self, emails: Vec<String>) -> Result<Uuid, GameError> {
        // Validate player count against configuration
        let player_count = emails.len();
        if player_count < self.config.min_players as usize
            || player_count > self.config.max_players as usize
        {
            return Err(GameError::InvalidPlayerCount {
                min: self.config.min_players,
                max: self.config.max_players,
                provided: player_count,
            });
        }

        // Create the game
        let game = Game::new(emails)?;
        let game_id = game.id;

        // Store the game
        let mut games = self.games.lock().unwrap();
        games.insert(game_id, game);

        tracing::info!(game_id = %game_id, player_count = player_count, "Game created");

        Ok(game_id)
    }

    /// Draws a card for a player in a game
    #[tracing::instrument(skip(self), fields(game_id, player_email))]
    pub fn draw_card(
        &self,
        game_id: Uuid,
        email: &str,
    ) -> Result<DrawCardResponse, GameError> {
        let mut games = self.games.lock().unwrap();
        let game = games.get_mut(&game_id).ok_or(GameError::GameNotFound)?;

        let card = game.draw_card(email)?;
        let player = game.players.get(email).ok_or(GameError::PlayerNotInGame)?;

        tracing::debug!(
            game_id = %game_id,
            player_email = email,
            card = ?card,
            "Card drawn"
        );

        Ok(DrawCardResponse {
            card,
            current_points: player.points,
            busted: player.busted,
            cards_remaining: game.available_cards.len(),
            cards_history: player.cards_history.clone(),
        })
    }

    /// Sets the value of an Ace card for a player
    #[tracing::instrument(skip(self), fields(game_id, player_email))]
    pub fn set_ace_value(
        &self,
        game_id: Uuid,
        email: &str,
        card_id: Uuid,
        as_eleven: bool,
    ) -> Result<PlayerStateResponse, GameError> {
        let mut games = self.games.lock().unwrap();
        let game = games.get_mut(&game_id).ok_or(GameError::GameNotFound)?;

        game.set_ace_value(email, card_id, as_eleven)?;
        let player = game.players.get(email).ok_or(GameError::PlayerNotInGame)?;

        Ok(PlayerStateResponse {
            points: player.points,
            busted: player.busted,
        })
    }

    /// Gets the current state of a game
    #[tracing::instrument(skip(self), fields(game_id))]
    pub fn get_game_state(&self, game_id: Uuid) -> Result<GameStateResponse, GameError> {
        let games = self.games.lock().unwrap();
        let game = games.get(&game_id).ok_or(GameError::GameNotFound)?;

        let mut players: HashMap<String, PlayerInfo> = HashMap::new();
        for (email, player) in &game.players {
            players.insert(
                email.clone(),
                PlayerInfo {
                    points: player.points,
                    cards_history: player.cards_history.clone(),
                    busted: player.busted,
                },
            );
        }

        Ok(GameStateResponse {
            players,
            cards_in_deck: game.available_cards.len(),
            finished: game.finished,
        })
    }

    /// Finishes a game and returns the results
    #[tracing::instrument(skip(self), fields(game_id))]
    pub fn finish_game(&self, game_id: Uuid) -> Result<GameResult, GameError> {
        let mut games = self.games.lock().unwrap();
        let game = games.get_mut(&game_id).ok_or(GameError::GameNotFound)?;

        game.finish_game();
        let results = game.calculate_results();

        tracing::info!(
            game_id = %game_id,
            winner = ?results.winner,
            highest_score = results.highest_score,
            "Game finished"
        );

        Ok(results)
    }
}
