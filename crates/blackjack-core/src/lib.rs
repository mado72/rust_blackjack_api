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

/// Represents a user in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

impl User {
    /// Creates a new user with the given email and password hash
    pub fn new(email: String, password_hash: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            email,
            password_hash,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
        }
    }
}

/// Status of a game invitation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum InvitationStatus {
    Pending,
    Accepted,
    Declined,
    Expired,
}

/// Represents a game invitation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameInvitation {
    pub id: Uuid,
    pub game_id: Uuid,
    pub inviter_id: Uuid,
    pub invitee_email: String,
    pub status: InvitationStatus,
    pub created_at: String,
    pub expires_at: String,
}

impl GameInvitation {
    /// Creates a new game invitation with expiration based on game's enrollment timeout
    pub fn new(
        game_id: Uuid,
        inviter_id: Uuid,
        invitee_email: String,
        game_enrollment_expires_at: String,
    ) -> Self {
        let created_at = chrono::Utc::now();

        Self {
            id: Uuid::new_v4(),
            game_id,
            inviter_id,
            invitee_email,
            status: InvitationStatus::Pending,
            created_at: created_at.to_rfc3339(),
            expires_at: game_enrollment_expires_at,
        }
    }

    /// Checks if the invitation has expired
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now();
        match chrono::DateTime::parse_from_rfc3339(&self.expires_at) {
            Ok(expires_at) => now > expires_at,
            Err(_) => false,
        }
    }
}

/// State of a player in the game
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PlayerState {
    Active,
    Standing,
    Busted,
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
    pub state: PlayerState,
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
            state: PlayerState::Active,
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
            if card.name == "A" && let Some(&is_eleven) = self.ace_values.get(&card.id) && is_eleven {
                self.points += 10;
            }
        }
        self.busted = self.points > 21;
        if self.busted {
            self.state = PlayerState::Busted;
        }
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
    NotPlayerTurn,
    PlayerNotActive,
    PlayerAlreadyEnrolled,
    EnrollmentNotClosed,
    GameNotActive,
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
            GameError::NotPlayerTurn => write!(f, "It's not this player's turn"),
            GameError::PlayerNotActive => write!(f, "Player is not active (standing or busted)"),
            GameError::PlayerAlreadyEnrolled => write!(f, "Player is already enrolled in this game"),
            GameError::EnrollmentNotClosed => write!(f, "Cannot play until enrollment is closed"),
            GameError::GameNotActive => write!(f, "Game is not active"),
        }
    }
}

impl std::error::Error for GameError {}

/// Represents a game with multiple players
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub id: Uuid,
    pub creator_id: Uuid,
    pub players: HashMap<String, Player>,
    pub dealer: Player,
    pub available_cards: Vec<Card>,
    pub finished: bool,
    pub turn_order: Vec<String>,
    pub current_turn_index: usize,
    pub enrollment_timeout_seconds: u64,
    pub enrollment_start_time: String,
    pub enrollment_closed: bool,
    pub active: bool,
}

impl Game {
    /// Creates a new game with the creator automatically enrolled
    #[tracing::instrument]
    pub fn new(creator_id: Uuid, creator_email: String, enrollment_timeout_seconds: u64) -> Result<Self, GameError> {
        // Validate email is not empty
        if creator_email.trim().is_empty() {
            return Err(GameError::InvalidEmail);
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

        // Auto-enroll creator as first player
        let mut players = HashMap::new();
        players.insert(creator_email.clone(), Player::new(creator_email.clone()));
        
        let mut turn_order = Vec::new();
        turn_order.push(creator_email);
        
        let dealer = Player::new("dealer".to_string());

        Ok(Self {
            id: Uuid::new_v4(),
            creator_id,
            players,
            dealer,
            available_cards,
            finished: false,
            turn_order,
            current_turn_index: 0,
            enrollment_timeout_seconds,
            enrollment_start_time: chrono::Utc::now().to_rfc3339(),
            enrollment_closed: false,
            active: true,
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

        // Check if it's the player's turn
        if !self.can_player_act(email) {
            return Err(GameError::NotPlayerTurn);
        }

        let player = self.players.get_mut(email).ok_or(GameError::PlayerNotInGame)?;

        if player.busted {
            return Err(GameError::PlayerAlreadyBusted);
        }

        if player.state != PlayerState::Active {
            return Err(GameError::PlayerNotActive);
        }

        // Draw a random card from the deck
        let random_index = rand::rng().random_range(0..self.available_cards.len());
        let card = self.available_cards.remove(random_index);

        player.add_card(card.clone());

        // Advance turn after drawing
        self.advance_turn();

        // Check if game should auto-finish
        if self.check_auto_finish() {
            // All players finished, play dealer automatically
            self.play_dealer()?;
            self.finished = true;
        }

        Ok(card)
    }

    /// Adds a player to the game (from invitation acceptance)
    pub fn add_player(&mut self, email: String) -> Result<(), GameError> {
        if !self.active {
            return Err(GameError::GameNotActive);
        }

        if self.finished {
            return Err(GameError::GameAlreadyFinished);
        }

        if self.enrollment_closed {
            return Err(GameError::InvalidPlayerCount);
        }

        // Validate email is not empty
        if email.trim().is_empty() {
            return Err(GameError::InvalidEmail);
        }

        if self.players.contains_key(&email) {
            return Err(GameError::PlayerAlreadyEnrolled);
        }

        if self.players.len() >= 10 {
            return Err(GameError::InvalidPlayerCount);
        }

        self.players.insert(email.clone(), Player::new(email.clone()));
        self.turn_order.push(email);

        Ok(())
    }

    /// Checks if enrollment is still open (not closed and timeout not exceeded)
    pub fn is_enrollment_open(&self) -> bool {
        if self.enrollment_closed {
            return false;
        }

        if let Ok(start_time) = chrono::DateTime::parse_from_rfc3339(&self.enrollment_start_time) {
            let now = chrono::Utc::now();
            let start_time_utc = start_time.with_timezone(&chrono::Utc);
            let elapsed = (now - start_time_utc).num_seconds();
            return elapsed < self.enrollment_timeout_seconds as i64;
        }

        false
    }

    /// Checks if can enroll (space available and enrollment is open)
    pub fn can_enroll(&self) -> bool {
        self.is_enrollment_open() && self.players.len() < 10
    }

    /// Closes enrollment and finalizes turn order
    pub fn close_enrollment(&mut self) -> Result<(), GameError> {
        if self.finished {
            return Err(GameError::GameAlreadyFinished);
        }

        self.enrollment_closed = true;

        // Reset turn index to start
        self.current_turn_index = 0;

        Ok(())
    }

    /// Gets the enrollment expiration time
    pub fn get_enrollment_expires_at(&self) -> String {
        if let Ok(start_time) = chrono::DateTime::parse_from_rfc3339(&self.enrollment_start_time) {
            let expires_at = start_time + chrono::Duration::seconds(self.enrollment_timeout_seconds as i64);
            return expires_at.to_rfc3339();
        }
        String::new()
    }

    /// Gets the time remaining for enrollment in seconds
    pub fn get_enrollment_time_remaining(&self) -> i64 {
        if self.enrollment_closed {
            return 0;
        }

        if let Ok(start_time) = chrono::DateTime::parse_from_rfc3339(&self.enrollment_start_time) {
            let now = chrono::Utc::now();
            let start_time_utc = start_time.with_timezone(&chrono::Utc);
            let elapsed = (now - start_time_utc).num_seconds();
            let remaining = self.enrollment_timeout_seconds as i64 - elapsed;
            return std::cmp::max(0, remaining);
        }

        0
    }

    /// Gets the email of the player whose turn it is
    pub fn get_current_player(&self) -> Option<&str> {
        if self.turn_order.is_empty() {
            return None;
        }
        self.turn_order.get(self.current_turn_index).map(|s| s.as_str())
    }

    /// Advances to the next active player's turn
    pub fn advance_turn(&mut self) {
        if self.turn_order.is_empty() {
            return;
        }

        let initial_index = self.current_turn_index;
        loop {
            self.current_turn_index = (self.current_turn_index + 1) % self.turn_order.len();
            
            // Check if we've gone full circle
            if self.current_turn_index == initial_index {
                break;
            }

            // Check if current player is active
            if let Some(email) = self.turn_order.get(self.current_turn_index)
                && let Some(player) = self.players.get(email)
                && player.state == PlayerState::Active
            {
                break;
            }
        }
    }

    /// Checks if the specified player can act (it's their turn and they're active and enrollment is closed)
    pub fn can_player_act(&self, email: &str) -> bool {
        if !self.enrollment_closed {
            return false;
        }

        if let Some(current_email) = self.get_current_player()
            && current_email == email
            && let Some(player) = self.players.get(email)
        {
            return player.state == PlayerState::Active;
        }
        false
    }

    /// Marks a player as standing (done playing)
    #[tracing::instrument(skip(self))]
    pub fn stand(&mut self, email: &str) -> Result<(), GameError> {
        if self.finished {
            return Err(GameError::GameAlreadyFinished);
        }

        // Check if it's the player's turn
        if !self.can_player_act(email) {
            return Err(GameError::NotPlayerTurn);
        }

        let player = self.players.get_mut(email).ok_or(GameError::PlayerNotInGame)?;

        if player.state != PlayerState::Active {
            return Err(GameError::PlayerNotActive);
        }

        player.state = PlayerState::Standing;

        // Advance turn after standing
        self.advance_turn();

        // Check if game should auto-finish
        if self.check_auto_finish() {
            // All players finished, play dealer automatically
            self.play_dealer()?;
            self.finished = true;
        }

        Ok(())
    }

    /// Checks if all players have finished playing (stood or busted)
    pub fn check_auto_finish(&self) -> bool {
        if self.players.is_empty() {
            return false;
        }

        self.players.values().all(|player| {
            player.state == PlayerState::Standing || player.state == PlayerState::Busted
        })
    }

    /// Plays the dealer's turn (draws until reaching 17 or higher)
    /// Should be called after all players have finished
    #[tracing::instrument(skip(self))]
    pub fn play_dealer(&mut self) -> Result<(), GameError> {
        if self.finished {
            return Err(GameError::GameAlreadyFinished);
        }

        // Dealer draws until reaching 17 or busting
        while self.dealer.points < 17 && !self.dealer.busted {
            if self.available_cards.is_empty() {
                return Err(GameError::DeckEmpty);
            }

            let random_index = rand::rng().random_range(0..self.available_cards.len());
            let card = self.available_cards.remove(random_index);
            self.dealer.add_card(card);
        }

        // Mark dealer as standing if not busted
        if !self.dealer.busted {
            self.dealer.state = PlayerState::Standing;
        }

        Ok(())
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

        // Add dealer to summaries
        all_players.insert(
            "dealer".to_string(),
            PlayerSummary {
                points: self.dealer.points,
                cards_count: self.dealer.cards_history.len(),
                busted: self.dealer.busted,
            },
        );

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

        // Dealer score (0 if busted)
        let dealer_score = if self.dealer.busted { 0 } else { self.dealer.points };

        // Find winner(s) - players who beat the dealer
        for (email, player) in &self.players {
            if !player.busted {
                // Player didn't bust
                if dealer_score == 0 {
                    // Dealer busted, all non-busted players win
                    if player.points == highest_score && highest_score > 0 {
                        tied_players.push(email.clone());
                    } else if player.points > highest_score {
                        highest_score = player.points;
                        winner = Some(email.clone());
                        tied_players.clear();
                    }
                } else if player.points > dealer_score {
                    // Player beat dealer
                    if player.points == highest_score && highest_score > 0 {
                        tied_players.push(email.clone());
                    } else if player.points > highest_score {
                        highest_score = player.points;
                        winner = Some(email.clone());
                        tied_players.clear();
                    }
                } else if player.points == dealer_score {
                    // Push (tie with dealer) - not counted as win
                    continue;
                }
                // else: player lost to dealer, skip
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
