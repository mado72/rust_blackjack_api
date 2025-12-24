use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Suits available in the deck
const SUITS: [&str; 4] = ["Hearts", "Diamonds", "Clubs", "Spades"];

/// Card types with their base values
const CARD_TYPES: [(&str, u8); 13] = [
    ("A", 1),
    ("2", 2),
    ("3", 3),
    ("4", 4),
    ("5", 5),
    ("6", 6),
    ("7", 7),
    ("8", 8),
    ("9", 9),
    ("10", 10),
    ("J", 10),
    ("Q", 10),
    ("K", 10),
];

/// Represents a single card in the game
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Card {
    pub id: Uuid,
    pub name: String,
    pub value: u8,
    pub suit: String,
}

/// Represents a player in the game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub email: String,
    pub points: u8,
    pub cards_history: Vec<Card>,
    /// Maps card_id to is_eleven (true = 11 points, false = 1 point)
    pub ace_values: HashMap<Uuid, bool>,
    pub busted: bool,
}

impl Player {
    /// Creates a new player with the given email
    pub fn new(email: String) -> Self {
        Self {
            email,
            points: 0,
            cards_history: Vec::new(),
            ace_values: HashMap::new(),
            busted: false,
        }
    }

    /// Adds a card to the player's hand and recalculates points
    pub fn add_card(&mut self, card: Card) {
        // If it's an Ace, default to counting as 1 (false in ace_values)
        if card.name == "A" {
            self.ace_values.insert(card.id, false);
        }
        self.cards_history.push(card);
        self.recalculate_points();
    }

    /// Recalculates the player's total points
    pub fn recalculate_points(&mut self) {
        self.points = 0;
        for card in &self.cards_history {
            self.points += card.value;
            // Add 10 extra points if this Ace is counted as 11
            if card.name == "A" {
                if let Some(&is_eleven) = self.ace_values.get(&card.id) {
                    if is_eleven {
                        self.points += 10;
                    }
                }
            }
        }
        self.busted = self.points > 21;
    }
}

/// Summary information about a player
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSummary {
    pub points: u8,
    pub cards_count: usize,
    pub busted: bool,
}

/// Result of a finished game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameResult {
    pub winner: Option<String>,
    pub tied_players: Vec<String>,
    pub highest_score: u8,
    pub all_players: HashMap<String, PlayerSummary>,
}

/// Errors that can occur during game operations
#[derive(Debug, Clone, PartialEq)]
pub enum GameError {
    GameNotFound,
    PlayerNotInGame,
    PlayerAlreadyBusted,
    InvalidPlayerCount,
    InvalidEmail,
    DeckEmpty,
    GameAlreadyFinished,
    CardNotFound,
    NotAnAce,
}

impl std::fmt::Display for GameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameError::GameNotFound => write!(f, "Game not found"),
            GameError::PlayerNotInGame => write!(f, "Player not in this game"),
            GameError::PlayerAlreadyBusted => write!(f, "Player already busted"),
            GameError::InvalidPlayerCount => write!(f, "Invalid player count (must be 1-10)"),
            GameError::InvalidEmail => write!(f, "Invalid email (cannot be empty)"),
            GameError::DeckEmpty => write!(f, "No more cards in the deck"),
            GameError::GameAlreadyFinished => write!(f, "Game has already finished"),
            GameError::CardNotFound => write!(f, "Card not found in player's hand"),
            GameError::NotAnAce => write!(f, "Can only change value of Ace cards"),
        }
    }
}

impl std::error::Error for GameError {}

/// Represents a game with multiple players
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub id: Uuid,
    pub players: HashMap<String, Player>,
    pub available_cards: Vec<Card>,
    pub finished: bool,
}

impl Game {
    /// Creates a new game with the specified players
    #[tracing::instrument]
    pub fn new(player_emails: Vec<String>) -> Result<Self, GameError> {
        // Validate player count
        if player_emails.is_empty() || player_emails.len() > 10 {
            return Err(GameError::InvalidPlayerCount);
        }

        // Validate emails are non-empty and unique
        let mut seen_emails = std::collections::HashSet::new();
        for email in &player_emails {
            if email.trim().is_empty() {
                return Err(GameError::InvalidEmail);
            }
            if !seen_emails.insert(email.clone()) {
                return Err(GameError::InvalidEmail);
            }
        }

        // Initialize 52-card deck (4 of each card type across 4 suits)
        let mut available_cards = Vec::new();
        for suit in SUITS.iter() {
            for (name, value) in CARD_TYPES.iter() {
                available_cards.push(Card {
                    id: Uuid::new_v4(),
                    name: name.to_string(),
                    value: *value,
                    suit: suit.to_string(),
                });
            }
        }

        // Initialize players
        let mut players = HashMap::new();
        for email in player_emails {
            players.insert(email.clone(), Player::new(email));
        }

        Ok(Self {
            id: Uuid::new_v4(),
            players,
            available_cards,
            finished: false,
        })
    }

    /// Draws a card for the specified player
    #[tracing::instrument(skip(self))]
    pub fn draw_card(&mut self, email: &str) -> Result<Card, GameError> {
        if self.finished {
            return Err(GameError::GameAlreadyFinished);
        }

        if self.available_cards.is_empty() {
            return Err(GameError::DeckEmpty);
        }

        let player = self.players.get_mut(email).ok_or(GameError::PlayerNotInGame)?;

        if player.busted {
            return Err(GameError::PlayerAlreadyBusted);
        }

        // Draw a random card from the deck
        let random_index = rand::rng().random_range(0..self.available_cards.len());
        let card = self.available_cards.remove(random_index);

        player.add_card(card.clone());

        Ok(card)
    }

    /// Sets the value of an Ace card for a player
    #[tracing::instrument(skip(self))]
    pub fn set_ace_value(
        &mut self,
        email: &str,
        card_id: Uuid,
        as_eleven: bool,
    ) -> Result<(), GameError> {
        if self.finished {
            return Err(GameError::GameAlreadyFinished);
        }

        let player = self.players.get_mut(email).ok_or(GameError::PlayerNotInGame)?;

        // Verify the card exists in player's hand
        let card = player
            .cards_history
            .iter()
            .find(|c| c.id == card_id)
            .ok_or(GameError::CardNotFound)?;

        // Verify it's an Ace
        if card.name != "A" {
            return Err(GameError::NotAnAce);
        }

        // Update the Ace value
        player.ace_values.insert(card_id, as_eleven);
        player.recalculate_points();

        Ok(())
    }

    /// Marks the game as finished
    pub fn finish_game(&mut self) {
        self.finished = true;
    }

    /// Calculates the game results
    pub fn calculate_results(&self) -> GameResult {
        let mut winner: Option<String> = None;
        let mut highest_score: u8 = 0;
        let mut tied_players: Vec<String> = Vec::new();
        let mut all_players: HashMap<String, PlayerSummary> = HashMap::new();

        // Build player summaries
        for (email, player) in &self.players {
            all_players.insert(
                email.clone(),
                PlayerSummary {
                    points: player.points,
                    cards_count: player.cards_history.len(),
                    busted: player.busted,
                },
            );
        }

        // Find winner(s) - based on determine_winner logic from CLI
        for (email, player) in &self.players {
            if player.points <= 21 {
                if player.points == highest_score && highest_score > 0 {
                    tied_players.push(email.clone());
                } else if player.points > highest_score {
                    highest_score = player.points;
                    winner = Some(email.clone());
                    tied_players.clear();
                }
            }
        }

        // If there are tied players, add the original winner to the list
        if !tied_players.is_empty() {
            if let Some(winner_email) = &winner {
                tied_players.insert(0, winner_email.clone());
            }
            winner = None; // Clear single winner if there's a tie
        }

        GameResult {
            winner,
            tied_players,
            highest_score,
            all_players,
        }
    }
}
