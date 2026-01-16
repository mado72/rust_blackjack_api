use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Password hashing and verification module
pub mod password;

/// Email and password validation module
pub mod validation;

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
    /// Argon2id password hash - NEVER store plaintext passwords
    pub password_hash: String,
    /// Account status - false means account is suspended
    #[serde(default = "default_active")]
    pub is_active: bool,
    /// Last successful login timestamp (ISO 8601 format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_login: Option<String>,
    /// Account creation timestamp (ISO 8601 format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// Player statistics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats: Option<UserStats>,
}

fn default_active() -> bool {
    true
}

/// Player statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserStats {
    pub games_played: u32,
    pub games_won: u32,
    pub games_lost: u32,
    pub games_tied: u32,
    pub total_points: u64,
    pub highest_score: u8,
    pub times_busted: u32,
}

impl UserStats {
    /// Creates new empty stats
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a game outcome
    pub fn record_game(&mut self, outcome: &PlayerOutcome, points: u8) {
        self.games_played += 1;
        self.total_points += points as u64;
        if points > self.highest_score {
            self.highest_score = points;
        }

        match outcome {
            PlayerOutcome::Won => self.games_won += 1,
            PlayerOutcome::Lost => self.games_lost += 1,
            PlayerOutcome::Push => self.games_tied += 1,
            PlayerOutcome::Busted => {
                self.games_lost += 1;
                self.times_busted += 1;
            }
        }
    }

    /// Calculate win rate as percentage
    pub fn win_rate(&self) -> f32 {
        if self.games_played == 0 {
            0.0
        } else {
            (self.games_won as f32 / self.games_played as f32) * 100.0
        }
    }

    /// Calculate average points per game
    pub fn average_points(&self) -> f32 {
        if self.games_played == 0 {
            0.0
        } else {
            self.total_points as f32 / self.games_played as f32
        }
    }
}

impl User {
    /// Creates a new user with the given email and password hash
    ///
    /// # Arguments
    ///
    /// * `email` - User's email address (must be validated before calling)
    /// * `password_hash` - Argon2id password hash (must be hashed before calling)
    ///
    /// # Note
    ///
    /// This function does NOT validate the email or hash the password.
    /// Use `validation::validate_email()` and `password::hash_password()` first.
    pub fn new(email: String, password_hash: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            email,
            password_hash,
            is_active: true,
            last_login: None,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            stats: Some(UserStats::new()),
        }
    }

    /// Updates the last login timestamp to current time
    pub fn update_last_login(&mut self) {
        self.last_login = Some(chrono::Utc::now().to_rfc3339());
    }

    /// Checks if the user account is active
    pub fn is_account_active(&self) -> bool {
        self.is_active
    }

    /// Deactivates the user account
    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    /// Activates the user account
    pub fn activate(&mut self) {
        self.is_active = true;
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
            if card.name == "A"
                && let Some(&is_eleven) = self.ace_values.get(&card.id)
                && is_eleven
            {
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

/// Player outcome in a finished game
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PlayerOutcome {
    Won,
    Lost,
    Push,
    Busted,
}

/// Detailed result information for a player
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerResult {
    pub points: u8,
    pub cards_count: usize,
    pub busted: bool,
    pub outcome: PlayerOutcome,
}

/// Result of a finished game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameResult {
    /// Email of the single winner (if no ties)
    pub winner: Option<String>,
    /// List of players who tied for the win
    pub tied_players: Vec<String>,
    /// Highest non-busted score
    pub highest_score: u8,
    /// All player summaries (including dealer)
    pub all_players: HashMap<String, PlayerSummary>,
    /// Detailed results for each player
    pub player_results: HashMap<String, PlayerResult>,
    /// Dealer's final points
    pub dealer_points: u8,
    /// Whether dealer busted
    pub dealer_busted: bool,
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
    /// User does not have permission to perform this action
    InsufficientPermissions,
    /// User is not a participant in the game
    NotAParticipant,
    /// Cannot kick the game creator
    CannotKickCreator,
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
            GameError::PlayerAlreadyEnrolled => {
                write!(f, "Player is already enrolled in this game")
            }
            GameError::EnrollmentNotClosed => write!(f, "Cannot play until enrollment is closed"),
            GameError::GameNotActive => write!(f, "Game is not active"),
            GameError::InsufficientPermissions => {
                write!(f, "User does not have permission to perform this action")
            }
            GameError::NotAParticipant => write!(f, "User is not a participant in this game"),
            GameError::CannotKickCreator => write!(f, "Cannot kick the game creator"),
        }
    }
}

impl std::error::Error for GameError {}

/// Game role types - defines participant roles in a game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GameRole {
    /// Game creator - has all permissions
    Creator,
    /// Regular player - limited to own actions
    Player,
    /// Spectator - future feature, read-only access
    Spectator,
}

/// Game permissions - specific actions that can be performed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamePermission {
    /// Invite other players to join the game
    InvitePlayers,
    /// Remove players from the game
    KickPlayers,
    /// Manually close enrollment to start the game
    CloseEnrollment,
    /// Manually finish the game (auto-finish still works)
    FinishGame,
    /// Modify game settings
    ModifySettings,
}

impl GameRole {
    /// Checks if this role has the specified permission
    ///
    /// # Permissions by role:
    ///
    /// - **Creator**: All permissions
    /// - **Player**: None (only their own actions like draw, stand)
    /// - **Spectator**: None (read-only, future feature)
    ///
    /// # Examples
    ///
    /// ```
    /// use blackjack_core::{GameRole, GamePermission};
    ///
    /// assert!(GameRole::Creator.has_permission(GamePermission::KickPlayers));
    /// assert!(!GameRole::Player.has_permission(GamePermission::KickPlayers));
    /// ```
    pub fn has_permission(&self, _permission: GamePermission) -> bool {
        match self {
            GameRole::Creator => true,    // Creator has all permissions
            GameRole::Player => false,    // Players only perform their own actions
            GameRole::Spectator => false, // Spectators are read-only
        }
    }

    /// Returns all permissions for this role
    pub fn permissions(&self) -> Vec<GamePermission> {
        match self {
            GameRole::Creator => vec![
                GamePermission::InvitePlayers,
                GamePermission::KickPlayers,
                GamePermission::CloseEnrollment,
                GamePermission::FinishGame,
                GamePermission::ModifySettings,
            ],
            GameRole::Player | GameRole::Spectator => vec![],
        }
    }
}

/// Represents a participant in a game with their role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameParticipant {
    pub user_id: Uuid,
    pub email: String,
    pub role: GameRole,
    pub joined_at: String,
}

impl GameParticipant {
    /// Creates a new game participant
    pub fn new(user_id: Uuid, email: String, role: GameRole) -> Self {
        Self {
            user_id,
            email,
            role,
            joined_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Represents a game with multiple players
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub id: Uuid,
    pub creator_id: Uuid,
    /// Participants with their roles (user_id -> GameParticipant)
    pub participants: HashMap<Uuid, GameParticipant>,
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
    pub fn new(
        creator_id: Uuid,
        creator_email: String,
        enrollment_timeout_seconds: u64,
    ) -> Result<Self, GameError> {
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

        // Initialize participants with creator as Creator role
        let mut participants = HashMap::new();
        participants.insert(
            creator_id,
            GameParticipant::new(creator_id, creator_email.clone(), GameRole::Creator),
        );

        let turn_order = vec![creator_email];

        let dealer = Player::new("dealer".to_string());

        Ok(Self {
            id: Uuid::new_v4(),
            creator_id,
            participants,
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

        // Check if enrollment is closed
        if !self.enrollment_closed {
            return Err(GameError::EnrollmentNotClosed);
        }

        // Check if it's the player's turn
        if !self.can_player_act(email) {
            return Err(GameError::NotPlayerTurn);
        }

        let player = self
            .players
            .get_mut(email)
            .ok_or(GameError::PlayerNotInGame)?;

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
            tracing::info!("All players finished - triggering automatic dealer play");
            // All players finished, play dealer automatically
            self.play_dealer()?;
            self.finished = true;
            tracing::info!("Game automatically finished after dealer play");
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

        self.players
            .insert(email.clone(), Player::new(email.clone()));
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
            let expires_at =
                start_time + chrono::Duration::seconds(self.enrollment_timeout_seconds as i64);
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

    /// Gets the role of a participant by user_id
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID to look up
    ///
    /// # Returns
    ///
    /// * `Some(GameRole)` - The user's role in this game
    /// * `None` - User is not a participant
    pub fn get_participant_role(&self, user_id: Uuid) -> Option<GameRole> {
        self.participants.get(&user_id).map(|p| p.role)
    }

    /// Checks if a user can perform a specific action
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user attempting the action
    /// * `permission` - The permission required
    ///
    /// # Returns
    ///
    /// * `true` - User has the required permission
    /// * `false` - User does not have permission or is not a participant
    ///
    /// # Examples
    ///
    /// ```
    /// use blackjack_core::{Game, GamePermission};
    /// use uuid::Uuid;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let creator_id = Uuid::new_v4();
    /// let game = Game::new(creator_id, "creator@example.com".to_string(), 300)?;
    ///
    /// // Creator can kick players
    /// assert!(game.can_user_perform(creator_id, GamePermission::KickPlayers));
    ///
    /// // Random user cannot
    /// let other_user = Uuid::new_v4();
    /// assert!(!game.can_user_perform(other_user, GamePermission::KickPlayers));
    /// # Ok(())
    /// # }
    /// ```
    pub fn can_user_perform(&self, user_id: Uuid, permission: GamePermission) -> bool {
        match self.get_participant_role(user_id) {
            Some(role) => role.has_permission(permission),
            None => false,
        }
    }

    /// Checks if a user is the game creator
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID to check
    ///
    /// # Returns
    ///
    /// * `true` - User is the creator
    /// * `false` - User is not the creator
    pub fn is_creator(&self, user_id: Uuid) -> bool {
        self.creator_id == user_id
    }

    /// Checks if a user is a participant in the game
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID to check
    ///
    /// # Returns
    ///
    /// * `true` - User is a participant
    /// * `false` - User is not a participant
    pub fn is_participant(&self, user_id: Uuid) -> bool {
        self.participants.contains_key(&user_id)
    }

    /// Adds a participant to the game (when enrolling)
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user's ID
    /// * `email` - The user's email
    ///
    /// # Note
    ///
    /// Players are added with `GameRole::Player` role.
    /// Only the creator gets `GameRole::Creator`.
    pub fn add_participant(&mut self, user_id: Uuid, email: String) {
        self.participants.insert(
            user_id,
            GameParticipant::new(user_id, email, GameRole::Player),
        );
    }

    /// Gets the email of the player whose turn it is
    pub fn get_current_player(&self) -> Option<&str> {
        if self.turn_order.is_empty() {
            return None;
        }
        self.turn_order
            .get(self.current_turn_index)
            .map(|s| s.as_str())
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

        // Check if enrollment is closed
        if !self.enrollment_closed {
            return Err(GameError::EnrollmentNotClosed);
        }

        // Check if it's the player's turn
        if !self.can_player_act(email) {
            return Err(GameError::NotPlayerTurn);
        }

        let player = self
            .players
            .get_mut(email)
            .ok_or(GameError::PlayerNotInGame)?;

        if player.state != PlayerState::Active {
            return Err(GameError::PlayerNotActive);
        }

        player.state = PlayerState::Standing;

        // Advance turn after standing
        self.advance_turn();

        // Check if game should auto-finish
        if self.check_auto_finish() {
            tracing::info!("All players finished after stand - triggering automatic dealer play");
            // All players finished, play dealer automatically
            self.play_dealer()?;
            self.finished = true;
            tracing::info!("Game automatically finished after dealer play");
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

    /// Plays the dealer's turn automatically
    ///
    /// The dealer follows standard blackjack rules:
    /// - Draws cards until reaching 17 or higher
    /// - Stops at 17-21 (soft 17 is treated as 17)
    /// - Busts if exceeding 21
    ///
    /// This method is automatically called when all players have finished
    /// their turns (either by standing or busting).
    ///
    /// # Returns
    /// - `Ok(())` if dealer played successfully
    /// - `Err(GameError::GameAlreadyFinished)` if game is already finished
    /// - `Err(GameError::DeckEmpty)` if deck runs out of cards
    ///
    /// # Example Flow
    /// ```text
    /// Dealer starts with 0 points
    /// Draws cards: 7, 8 (total: 15) - continues
    /// Draws card: 5 (total: 20) - stops (>= 17)
    /// Final state: Standing with 20 points
    /// ```
    #[tracing::instrument(skip(self))]
    pub fn play_dealer(&mut self) -> Result<(), GameError> {
        if self.finished {
            return Err(GameError::GameAlreadyFinished);
        }

        tracing::info!("Dealer starting turn with {} points", self.dealer.points);

        // Dealer draws until reaching 17 or busting
        while self.dealer.points < 17 && !self.dealer.busted {
            if self.available_cards.is_empty() {
                tracing::warn!("Deck empty during dealer play");
                return Err(GameError::DeckEmpty);
            }

            let random_index = rand::rng().random_range(0..self.available_cards.len());
            let card = self.available_cards.remove(random_index);

            tracing::debug!(
                "Dealer draws {} of {} (value: {})",
                card.name,
                card.suit,
                card.value
            );

            self.dealer.add_card(card);

            tracing::debug!("Dealer now has {} points", self.dealer.points);
        }

        // Mark dealer as standing if not busted
        if !self.dealer.busted {
            self.dealer.state = PlayerState::Standing;
            tracing::info!(
                "Dealer stands with {} points (cards: {})",
                self.dealer.points,
                self.dealer.cards_history.len()
            );
        } else {
            tracing::info!(
                "Dealer busted with {} points (cards: {})",
                self.dealer.points,
                self.dealer.cards_history.len()
            );
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

        let player = self
            .players
            .get_mut(email)
            .ok_or(GameError::PlayerNotInGame)?;

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
        let mut player_results: HashMap<String, PlayerResult> = HashMap::new();

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
        let dealer_score = if self.dealer.busted {
            0
        } else {
            self.dealer.points
        };

        // Calculate individual player results and find winner(s)
        for (email, player) in &self.players {
            let outcome = if player.busted {
                PlayerOutcome::Busted
            } else if dealer_score == 0 {
                // Dealer busted, all non-busted players win
                PlayerOutcome::Won
            } else if player.points > dealer_score {
                // Player beat dealer
                PlayerOutcome::Won
            } else if player.points == dealer_score {
                // Push (tie with dealer)
                PlayerOutcome::Push
            } else {
                // Player lost to dealer
                PlayerOutcome::Lost
            };

            player_results.insert(
                email.clone(),
                PlayerResult {
                    points: player.points,
                    cards_count: player.cards_history.len(),
                    busted: player.busted,
                    outcome,
                },
            );

            // Track highest winning score for backward compatibility
            if !player.busted {
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
            player_results,
            dealer_points: self.dealer.points,
            dealer_busted: self.dealer.busted,
        }
    }
}
