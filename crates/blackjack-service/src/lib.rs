use blackjack_core::{
    Card, Game, GameError as CoreGameError, GameInvitation, GameResult, InvitationStatus, User,
    password, validation,
};
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
    #[error("Game is full")]
    GameFull,
    #[error("Enrollment is closed")]
    EnrollmentClosed,
    #[error("Invalid email: {0}")]
    InvalidEmail(String),
    #[error("No more cards in the deck")]
    DeckEmpty,
    #[error("Game has already finished")]
    GameAlreadyFinished,
    #[error("User not found")]
    UserNotFound,
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Invitation not found")]
    InvitationNotFound,
    #[error("Invitation expired")]
    InvitationExpired,
    #[error("Invalid timeout (must be between 1 and {max} seconds)")]
    InvalidTimeout { max: u64 },
    #[error("Not player's turn")]
    NotPlayerTurn,
    #[error("Player not active")]
    PlayerNotActive,
    #[error("Not game creator")]
    NotGameCreator,
    #[error("Enrollment not closed")]
    EnrollmentNotClosed,
    #[error("Player already enrolled")]
    PlayerAlreadyEnrolled,
    #[error("Game not active")]
    GameNotActive,
    #[error("Weak password: {0}")]
    WeakPassword(String),
    #[error("Account is inactive")]
    AccountInactive,
    #[error("Insufficient permissions")]
    InsufficientPermissions,
    #[error("Account is locked due to too many failed login attempts")]
    AccountLocked,
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Password hashing failed: {0}")]
    PasswordHashError(String),
    #[error("Core game error: {0}")]
    CoreError(#[from] CoreGameError),
}

/// Configuration for the game service
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub max_players: u8,
    pub min_players: u8,
}

/// Configuration for invitation management
#[derive(Debug, Clone)]
pub struct InvitationConfig {
    pub default_timeout_seconds: u64,
    pub max_timeout_seconds: u64,
}

impl Default for InvitationConfig {
    fn default() -> Self {
        Self {
            default_timeout_seconds: 300, // 5 minutes
            max_timeout_seconds: 3600,    // 1 hour
        }
    }
}

impl InvitationConfig {
    /// Load configuration from environment variables with defaults
    pub fn from_env() -> Self {
        let default_timeout_seconds =
            std::env::var("BLACKJACK_INVITATIONS_DEFAULT_TIMEOUT_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(300);

        let max_timeout_seconds = std::env::var("BLACKJACK_INVITATIONS_MAX_TIMEOUT_SECONDS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3600);

        Self {
            default_timeout_seconds,
            max_timeout_seconds,
        }
    }
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
    pub current_turn_player: Option<String>,
    pub turn_order: Vec<String>,
    pub enrollment_timeout_seconds: u64,
    pub enrollment_closes_at: String,
    pub time_remaining_seconds: i64,
    pub enrollment_closed: bool,
}

/// Information about a game in enrollment phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameInfo {
    pub game_id: Uuid,
    pub creator_id: Uuid,
    pub enrolled_count: u64,
    pub max_players: u64,
    pub enrollment_timeout_seconds: u64,
    pub time_remaining_seconds: i64,
    pub enrollment_closes_at: String,
}

/// Information about an invitation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvitationInfo {
    pub id: Uuid,
    pub game_id: Uuid,
    pub inviter_email: String,
    pub invitee_email: String,
    pub status: String,
    pub timeout_seconds: u64,
    pub expires_at: String,
    pub expires_in_seconds: i64,
}

/// User management service
pub struct UserService {
    users: Arc<Mutex<HashMap<Uuid, User>>>,
    email_index: Arc<Mutex<HashMap<String, Uuid>>>,
}

impl UserService {
    /// Creates a new user service
    pub fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(HashMap::new())),
            email_index: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Registers a new user with secure password hashing
    ///
    /// # Security
    ///
    /// - Validates email format
    /// - Validates password complexity (min 8 chars, uppercase, lowercase, digit, special char)
    /// - Hashes password using Argon2id with OWASP recommended parameters
    /// - Prevents duplicate email registration
    ///
    /// # Arguments
    ///
    /// * `email` - User's email address (must be unique and valid format)
    /// * `password` - Plaintext password (will be hashed, never stored as plaintext)
    ///
    /// # Returns
    ///
    /// * `Ok(Uuid)` - The new user's ID
    /// * `Err(GameError)` - Validation or registration error
    #[tracing::instrument(skip(self, password))]
    pub fn register(&self, email: String, password: String) -> Result<Uuid, GameError> {
        // Validate email format
        validation::validate_email(&email)
            .map_err(|e| GameError::ValidationError(e.to_string()))?;

        // Validate password complexity
        validation::validate_password(&password)
            .map_err(|e| GameError::WeakPassword(e.to_string()))?;

        let mut email_index = self.email_index.lock().unwrap();

        // Check if user already exists
        if email_index.contains_key(&email) {
            tracing::warn!(email = %email, "Registration failed: email already exists");
            return Err(GameError::UserAlreadyExists);
        }

        // Hash password using Argon2id
        let password_hash = password::hash_password(&password)
            .map_err(|e| GameError::PasswordHashError(e.to_string()))?;

        let user = User::new(email.clone(), password_hash);
        let user_id = user.id;

        let mut users = self.users.lock().unwrap();
        users.insert(user_id, user);
        email_index.insert(email.clone(), user_id);

        tracing::info!(user_id = %user_id, email = %email, "User registered successfully");

        Ok(user_id)
    }

    /// Authenticates a user with secure password verification
    ///
    /// # Security
    ///
    /// - Uses constant-time password comparison via Argon2id
    /// - Updates last_login timestamp on successful login
    /// - Checks account is_active status
    /// - Logs authentication attempts (success and failure)
    /// - Does not reveal whether email or password is incorrect
    ///
    /// # Arguments
    ///
    /// * `email` - User's email address
    /// * `password` - Plaintext password to verify
    ///
    /// # Returns
    ///
    /// * `Ok(User)` - Authenticated user with updated last_login
    /// * `Err(GameError)` - Authentication failure (invalid credentials or inactive account)
    #[tracing::instrument(skip(self, password))]
    pub fn login(&self, email: &str, password: &str) -> Result<User, GameError> {
        let email_index = self.email_index.lock().unwrap();
        let user_id = email_index.get(email).ok_or_else(|| {
            tracing::warn!(email = %email, "Login failed: user not found");
            GameError::InvalidCredentials
        })?;

        let mut users = self.users.lock().unwrap();
        let user = users.get_mut(user_id).ok_or(GameError::UserNotFound)?;

        // Check if account is active
        if !user.is_account_active() {
            tracing::warn!(user_id = %user_id, email = %email, "Login failed: account inactive");
            return Err(GameError::AccountInactive);
        }

        // Verify password using constant-time comparison
        let password_valid =
            password::verify_password(password, &user.password_hash).map_err(|e| {
                tracing::error!(user_id = %user_id, "Password verification error: {}", e);
                GameError::PasswordHashError(e.to_string())
            })?;

        if !password_valid {
            tracing::warn!(user_id = %user_id, email = %email, "Login failed: incorrect password");
            return Err(GameError::InvalidCredentials);
        }

        // Update last login timestamp
        user.update_last_login();

        tracing::info!(user_id = %user_id, email = %email, "User logged in successfully");

        Ok(user.clone())
    }

    /// Gets a user by ID
    pub fn get_user(&self, user_id: Uuid) -> Result<User, GameError> {
        let users = self.users.lock().unwrap();
        users.get(&user_id).cloned().ok_or(GameError::UserNotFound)
    }

    /// Gets a user by email
    pub fn get_user_by_email(&self, email: &str) -> Result<User, GameError> {
        let email_index = self.email_index.lock().unwrap();
        let user_id = email_index.get(email).ok_or(GameError::UserNotFound)?;

        let users = self.users.lock().unwrap();
        users.get(user_id).cloned().ok_or(GameError::UserNotFound)
    }

    /// Changes a user's password
    ///
    /// # Security
    ///
    /// - Verifies old password before allowing change
    /// - Validates new password complexity
    /// - Hashes new password using Argon2id
    /// - Logs password change events
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user's ID
    /// * `old_password` - Current password for verification
    /// * `new_password` - New password to set
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Password changed successfully
    /// * `Err(GameError)` - Verification or validation error
    ///
    /// # Note
    ///
    /// After password change, all existing JWT tokens should be invalidated
    /// to force re-login (implemented at API layer).
    #[tracing::instrument(skip(self, old_password, new_password))]
    pub fn change_password(
        &self,
        user_id: Uuid,
        old_password: &str,
        new_password: &str,
    ) -> Result<(), GameError> {
        let mut users = self.users.lock().unwrap();
        let user = users.get_mut(&user_id).ok_or(GameError::UserNotFound)?;

        // Verify old password
        let old_password_valid = password::verify_password(old_password, &user.password_hash)
            .map_err(|e| {
                tracing::error!(user_id = %user_id, "Password verification error: {}", e);
                GameError::PasswordHashError(e.to_string())
            })?;

        if !old_password_valid {
            tracing::warn!(user_id = %user_id, "Password change failed: incorrect old password");
            return Err(GameError::InvalidCredentials);
        }

        // Validate new password complexity
        validation::validate_password(new_password)
            .map_err(|e| GameError::WeakPassword(e.to_string()))?;

        // Hash new password
        let new_password_hash = password::hash_password(new_password)
            .map_err(|e| GameError::PasswordHashError(e.to_string()))?;

        // Update password
        user.password_hash = new_password_hash;

        tracing::info!(user_id = %user_id, "Password changed successfully");

        Ok(())
    }

    /// Deactivates a user account (Milestone 8)
    ///
    /// Sets the account status to inactive, preventing login.
    /// Used for account suspension or administrative actions.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user to deactivate
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Account deactivated successfully
    /// * `Err(GameError::UserNotFound)` - User doesn't exist
    ///
    /// # Example
    ///
    /// ```ignore
    /// user_service.deactivate_account(user_id)?;
    /// // User can no longer login
    /// ```
    #[tracing::instrument(skip(self))]
    pub fn deactivate_account(&self, user_id: Uuid) -> Result<(), GameError> {
        let mut users = self.users.lock().unwrap();
        let user = users.get_mut(&user_id).ok_or(GameError::UserNotFound)?;

        user.deactivate();

        tracing::info!(user_id = %user_id, "Account deactivated");

        Ok(())
    }

    /// Activates a user account (Milestone 8)
    ///
    /// Sets the account status to active, allowing login.
    /// Used to restore previously deactivated accounts.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user to activate
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Account activated successfully
    /// * `Err(GameError::UserNotFound)` - User doesn't exist
    #[tracing::instrument(skip(self))]
    pub fn activate_account(&self, user_id: Uuid) -> Result<(), GameError> {
        let mut users = self.users.lock().unwrap();
        let user = users.get_mut(&user_id).ok_or(GameError::UserNotFound)?;

        user.activate();

        tracing::info!(user_id = %user_id, "Account activated");

        Ok(())
    }
}

impl Default for UserService {
    fn default() -> Self {
        Self::new()
    }
}

/// Invitation management service
pub struct InvitationService {
    invitations: Arc<Mutex<HashMap<Uuid, GameInvitation>>>,
    #[allow(dead_code)]
    config: InvitationConfig,
}

impl InvitationService {
    /// Creates a new invitation service
    pub fn new(config: InvitationConfig) -> Self {
        Self {
            invitations: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }

    /// Creates a new game invitation using the game's enrollment expiration time
    ///
    /// # Security (Milestone 8)
    ///
    /// Only game participants (creator or enrolled players) can invite others.
    /// Uses RBAC permission check to verify inviter has `InvitePlayers` permission.
    ///
    /// # Arguments
    ///
    /// * `game_id` - The game to invite to
    /// * `inviter_id` - User creating the invitation
    /// * `invitee_email` - Email of user being invited
    /// * `game_enrollment_expires_at` - Enrollment expiration from game
    /// * `games` - Reference to games HashMap for permission check
    ///
    /// # Errors
    ///
    /// - `InsufficientPermissions` if inviter is not a participant
    /// - `GameNotFound` if game doesn't exist
    #[tracing::instrument(skip(self, games))]
    pub fn create(
        &self,
        game_id: Uuid,
        inviter_id: Uuid,
        invitee_email: String,
        game_enrollment_expires_at: String,
        games: &std::sync::Arc<std::sync::Mutex<HashMap<Uuid, Game>>>,
    ) -> Result<Uuid, GameError> {
        use blackjack_core::GamePermission;

        // Check if inviter has permission
        let games_lock = games.lock().unwrap();
        let game = games_lock.get(&game_id).ok_or(GameError::GameNotFound)?;

        if !game.can_user_perform(inviter_id, GamePermission::InvitePlayers) {
            tracing::warn!(
                game_id = %game_id,
                inviter_id = %inviter_id,
                "Permission denied: user attempted to invite player"
            );
            return Err(GameError::InsufficientPermissions);
        }

        drop(games_lock); // Release lock before creating invitation

        let invitation = GameInvitation::new(
            game_id,
            inviter_id,
            invitee_email,
            game_enrollment_expires_at,
        );
        let invitation_id = invitation.id;

        let mut invitations = self.invitations.lock().unwrap();
        invitations.insert(invitation_id, invitation);

        tracing::info!(
            invitation_id = %invitation_id,
            game_id = %game_id,
            inviter_id = %inviter_id,
            "Invitation created"
        );

        Ok(invitation_id)
    }

    /// Accepts an invitation
    #[tracing::instrument(skip(self))]
    pub fn accept(&self, invitation_id: Uuid) -> Result<GameInvitation, GameError> {
        let mut invitations = self.invitations.lock().unwrap();
        let invitation = invitations
            .get_mut(&invitation_id)
            .ok_or(GameError::InvitationNotFound)?;

        // Check if expired
        if invitation.is_expired() {
            invitation.status = InvitationStatus::Expired;
            return Err(GameError::InvitationExpired);
        }

        invitation.status = InvitationStatus::Accepted;

        tracing::info!(invitation_id = %invitation_id, "Invitation accepted");

        Ok(invitation.clone())
    }

    /// Declines an invitation
    #[tracing::instrument(skip(self))]
    pub fn decline(&self, invitation_id: Uuid) -> Result<(), GameError> {
        let mut invitations = self.invitations.lock().unwrap();
        let invitation = invitations
            .get_mut(&invitation_id)
            .ok_or(GameError::InvitationNotFound)?;

        invitation.status = InvitationStatus::Declined;

        tracing::info!(invitation_id = %invitation_id, "Invitation declined");

        Ok(())
    }

    /// Gets all pending invitations for a user
    pub fn get_pending_for_user(&self, email: &str) -> Vec<InvitationInfo> {
        let mut invitations = self.invitations.lock().unwrap();
        let now = chrono::Utc::now();

        invitations
            .values_mut()
            .filter_map(|inv| {
                if inv.invitee_email == email && inv.status == InvitationStatus::Pending {
                    // Auto-expire if needed
                    if inv.is_expired() {
                        inv.status = InvitationStatus::Expired;
                        return None;
                    }

                    // Calculate expires_in_seconds
                    let expires_at = chrono::DateTime::parse_from_rfc3339(&inv.expires_at).ok()?;
                    let expires_at_utc = expires_at.with_timezone(&chrono::Utc);
                    let expires_in = (expires_at_utc - now).num_seconds();

                    Some(InvitationInfo {
                        id: inv.id,
                        game_id: inv.game_id,
                        inviter_email: inv.inviter_id.to_string(), // Use inviter_id but convert to string
                        invitee_email: inv.invitee_email.clone(),
                        status: format!("{:?}", inv.status).to_lowercase(),
                        timeout_seconds: 0, // No longer stored; calculated from game enrollment
                        expires_at: inv.expires_at.clone(),
                        expires_in_seconds: expires_in,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Cleans up expired invitations
    pub fn cleanup_expired(&self) -> usize {
        let mut invitations = self.invitations.lock().unwrap();
        let mut count = 0;

        for invitation in invitations.values_mut() {
            if invitation.status == InvitationStatus::Pending && invitation.is_expired() {
                invitation.status = InvitationStatus::Expired;
                count += 1;
            }
        }

        if count > 0 {
            tracing::debug!(count = count, "Expired invitations marked");
        }

        count
    }

    /// Gets an invitation by ID
    pub fn get_invitation(&self, invitation_id: Uuid) -> Result<GameInvitation, GameError> {
        let invitations = self.invitations.lock().unwrap();
        invitations
            .get(&invitation_id)
            .cloned()
            .ok_or(GameError::InvitationNotFound)
    }
}

/// Main game service managing multiple games
pub struct GameService {
    pub games: Arc<Mutex<HashMap<Uuid, Game>>>,
    user_service: Arc<UserService>,
    config: ServiceConfig,
}

impl GameService {
    /// Creates a new game service with the given configuration
    pub fn new(config: ServiceConfig, user_service: Arc<UserService>) -> Self {
        Self {
            games: Arc::new(Mutex::new(HashMap::new())),
            user_service,
            config,
        }
    }

    /// Creates a new game service with default configuration
    pub fn new_default() -> Self {
        let user_service = Arc::new(UserService::new());
        Self::new(ServiceConfig::default(), user_service)
    }

    /// Creates a new game with the specified creator and enrollment timeout
    /// The creator is automatically enrolled in the game
    /// Creator's email is retrieved from the user database
    #[tracing::instrument(skip(self), fields(game_id))]
    pub fn create_game(
        &self,
        creator_id: Uuid,
        enrollment_timeout_seconds: Option<u64>,
    ) -> Result<Uuid, GameError> {
        // Use provided timeout or default to 300 seconds
        let timeout = enrollment_timeout_seconds.unwrap_or(300);

        // Get creator's email from user service
        let creator = self.user_service.get_user(creator_id)?;
        let creator_email = creator.email;

        // Create game with creator automatically enrolled
        let game = Game::new(creator_id, creator_email.clone(), timeout)?;
        let game_id = game.id;

        // Store the game
        let mut games = self.games.lock().unwrap();
        games.insert(game_id, game);

        tracing::info!(game_id = %game_id, creator_id = %creator_id, creator_email = %creator_email, enrollment_timeout_seconds = timeout, "Game created with creator auto-enrolled");

        Ok(game_id)
    }

    /// Lists all open games (in enrollment phase)
    pub fn get_open_games(
        &self,
        exclude_user_id: Option<Uuid>,
    ) -> Result<Vec<GameInfo>, GameError> {
        let games = self.games.lock().unwrap();
        let now = chrono::Utc::now();

        let _ = exclude_user_id; // Reserved for future use when user-game relationship exists

        let open_games = games
            .values()
            .filter(|game| {
                // Game must be in enrollment phase and not finished
                if game.finished {
                    return false;
                }

                // Check if enrollment is still open
                if !game.is_enrollment_open() {
                    return false;
                }

                true
            })
            .map(|game| {
                let expires_at = game.get_enrollment_expires_at();
                let expires_at_parsed = chrono::DateTime::parse_from_rfc3339(&expires_at).ok();
                let _expires_in = expires_at_parsed
                    .map(|dt| (dt.with_timezone(&chrono::Utc) - now).num_seconds())
                    .unwrap_or(0);

                GameInfo {
                    game_id: game.id,
                    creator_id: game.creator_id,
                    enrolled_count: game.players.len() as u64,
                    max_players: 10,
                    enrollment_timeout_seconds: game.enrollment_timeout_seconds,
                    time_remaining_seconds: game.get_enrollment_time_remaining(),
                    enrollment_closes_at: expires_at.clone(),
                }
            })
            .collect();

        Ok(open_games)
    }

    /// Enrolls a player in a game
    #[tracing::instrument(skip(self), fields(game_id, user_id))]
    pub fn enroll_player(&self, game_id: Uuid, user_id: Uuid) -> Result<(), GameError> {
        let mut games = self.games.lock().unwrap();
        let game = games.get_mut(&game_id).ok_or(GameError::GameNotFound)?;

        // Check if enrollment is open
        if !game.is_enrollment_open() {
            return Err(GameError::EnrollmentClosed);
        }

        // Check if game is full
        if !game.can_enroll() {
            return Err(GameError::GameFull);
        }

        // Get user email from user service
        let user = self.user_service.get_user(user_id)?;
        let player_email = user.email.clone();

        // Add player - explicitly map core errors to service errors
        game.add_player(player_email.clone()).map_err(|e| match e {
            CoreGameError::GameNotActive => GameError::GameNotActive,
            CoreGameError::PlayerAlreadyEnrolled => GameError::PlayerAlreadyEnrolled,
            other => GameError::CoreError(other),
        })?;

        // Add to participants with Player role (M8: RBAC)
        game.add_participant(user_id, player_email.clone());

        tracing::info!(
            game_id = %game_id,
            user_id = %user_id,
            player_email = %player_email,
            enrolled_count = game.players.len(),
            "Player enrolled in game"
        );

        Ok(())
    }

    /// Closes enrollment for a game (only creator can do this)
    ///
    /// # Security (Milestone 8)
    ///
    /// Uses RBAC permission check. Only users with `CloseEnrollment` permission
    /// (i.e., the game creator) can close enrollment.
    ///
    /// # Errors
    ///
    /// - `InsufficientPermissions` if user doesn't have permission
    /// - `GameNotFound` if game doesn't exist
    #[tracing::instrument(skip(self), fields(game_id, user_id))]
    pub fn close_enrollment(&self, game_id: Uuid, user_id: Uuid) -> Result<Vec<String>, GameError> {
        use blackjack_core::GamePermission;

        let mut games = self.games.lock().unwrap();
        let game = games.get_mut(&game_id).ok_or(GameError::GameNotFound)?;

        // Check if user has permission to close enrollment
        if !game.can_user_perform(user_id, GamePermission::CloseEnrollment) {
            tracing::warn!(
                game_id = %game_id,
                user_id = %user_id,
                "Permission denied: user attempted to close enrollment"
            );
            return Err(GameError::InsufficientPermissions);
        }

        game.close_enrollment()?;

        tracing::info!(
            game_id = %game_id,
            user_id = %user_id,
            enrolled_count = game.players.len(),
            turn_order = ?game.turn_order,
            "Enrollment closed"
        );

        Ok(game.turn_order.clone())
    }

    /// Draws a card for a player in a game
    #[tracing::instrument(skip(self), fields(game_id, user_id))]
    pub fn draw_card(&self, game_id: Uuid, user_id: Uuid) -> Result<DrawCardResponse, GameError> {
        // Get user email from user service
        let user = self.user_service.get_user(user_id)?;
        let email = user.email;

        let mut games = self.games.lock().unwrap();
        let game = games.get_mut(&game_id).ok_or(GameError::GameNotFound)?;

        // Map core errors to service errors for proper HTTP status codes
        let card = game.draw_card(&email).map_err(|e| match e {
            CoreGameError::EnrollmentNotClosed => GameError::EnrollmentNotClosed,
            CoreGameError::NotPlayerTurn => GameError::NotPlayerTurn,
            CoreGameError::PlayerNotActive => GameError::PlayerNotActive,
            CoreGameError::GameAlreadyFinished => GameError::GameAlreadyFinished,
            CoreGameError::DeckEmpty => GameError::DeckEmpty,
            CoreGameError::PlayerAlreadyBusted => GameError::PlayerAlreadyBusted,
            CoreGameError::PlayerNotInGame => GameError::PlayerNotInGame,
            other => GameError::CoreError(other),
        })?;

        let player = game.players.get(&email).ok_or(GameError::PlayerNotInGame)?;

        tracing::debug!(
            game_id = %game_id,
            player_email = %email,
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
        user_id: Uuid,
        card_id: Uuid,
        as_eleven: bool,
    ) -> Result<PlayerStateResponse, GameError> {
        // Get user email from user service
        let user = self.user_service.get_user(user_id)?;
        let email = user.email;

        let mut games = self.games.lock().unwrap();
        let game = games.get_mut(&game_id).ok_or(GameError::GameNotFound)?;

        game.set_ace_value(&email, card_id, as_eleven)?;
        let player = game.players.get(&email).ok_or(GameError::PlayerNotInGame)?;

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
            current_turn_player: game.get_current_player().map(|s| s.to_string()),
            turn_order: game.turn_order.clone(),
            enrollment_timeout_seconds: game.enrollment_timeout_seconds,
            enrollment_closes_at: game.get_enrollment_expires_at(),
            time_remaining_seconds: game.get_enrollment_time_remaining(),
            enrollment_closed: game.enrollment_closed,
        })
    }

    /// Player stands (stops playing)
    #[tracing::instrument(skip(self), fields(game_id, user_id))]
    pub fn stand(&self, game_id: Uuid, user_id: Uuid) -> Result<GameStateResponse, GameError> {
        // Get user email from user service
        let user = self.user_service.get_user(user_id)?;
        let email = user.email;

        let mut games = self.games.lock().unwrap();
        let game = games.get_mut(&game_id).ok_or(GameError::GameNotFound)?;

        game.stand(&email)?;

        tracing::info!(
            game_id = %game_id,
            player_email = email,
            is_finished = game.finished,
            "Player stood"
        );

        // Build response
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
            current_turn_player: game.get_current_player().map(|s| s.to_string()),
            turn_order: game.turn_order.clone(),
            enrollment_timeout_seconds: game.enrollment_timeout_seconds,
            enrollment_closes_at: game.get_enrollment_expires_at(),
            time_remaining_seconds: game.get_enrollment_time_remaining(),
            enrollment_closed: game.enrollment_closed,
        })
    }

    /// Adds a player to a game (from invitation acceptance)
    #[tracing::instrument(skip(self), fields(game_id, user_id))]
    pub fn add_player_to_game(&self, game_id: Uuid, user_id: Uuid) -> Result<(), GameError> {
        // Get user email from user service
        let user = self.user_service.get_user(user_id)?;
        let email = user.email;

        let mut games = self.games.lock().unwrap();
        let game = games.get_mut(&game_id).ok_or(GameError::GameNotFound)?;

        game.add_player(email.clone())?;

        tracing::info!(
            game_id = %game_id,
            user_id = %user_id,
            player_email = %email,
            "Player added to game"
        );

        Ok(())
    }

    /// Removes a player from a game (only creator can do this)
    ///
    /// # Security (Milestone 8)
    ///
    /// - Only the game creator can kick players
    /// - Cannot kick the creator themselves
    /// - Can only kick during enrollment phase (before enrollment closes)
    ///
    /// # Arguments
    ///
    /// * `game_id` - The game ID
    /// * `kicker_id` - User attempting to kick
    /// * `player_id` - User to be kicked
    ///
    /// # Returns
    ///
    /// The email of the kicked player
    ///
    /// # Errors
    ///
    /// - `InsufficientPermissions` if kicker is not the creator
    /// - `CannotKickCreator` if attempting to kick the creator
    /// - `EnrollmentClosed` if enrollment has already closed
    /// - `PlayerNotInGame` if player is not enrolled
    /// - `GameNotFound` if game doesn't exist
    #[tracing::instrument(skip(self), fields(game_id, kicker_id, player_id))]
    pub fn kick_player(
        &self,
        game_id: Uuid,
        kicker_id: Uuid,
        player_id: Uuid,
    ) -> Result<String, GameError> {
        use blackjack_core::GamePermission;

        let mut games = self.games.lock().unwrap();
        let game = games.get_mut(&game_id).ok_or(GameError::GameNotFound)?;

        // Check if kicker has permission
        if !game.can_user_perform(kicker_id, GamePermission::KickPlayers) {
            tracing::warn!(
                game_id = %game_id,
                kicker_id = %kicker_id,
                "Permission denied: user attempted to kick player"
            );
            return Err(GameError::InsufficientPermissions);
        }

        // Cannot kick the creator
        if game.is_creator(player_id) {
            tracing::warn!(
                game_id = %game_id,
                kicker_id = %kicker_id,
                player_id = %player_id,
                "Cannot kick game creator"
            );
            return Err(GameError::CoreError(
                blackjack_core::GameError::CannotKickCreator,
            ));
        }

        // Can only kick during enrollment
        if game.enrollment_closed {
            return Err(GameError::EnrollmentClosed);
        }

        // Get player's email before removing
        let player_email = game
            .participants
            .get(&player_id)
            .map(|p| p.email.clone())
            .ok_or(GameError::PlayerNotInGame)?;

        // Remove from participants
        game.participants.remove(&player_id);

        // Remove from players HashMap
        game.players.remove(&player_email);

        // Remove from turn order
        game.turn_order.retain(|email| email != &player_email);

        tracing::info!(
            game_id = %game_id,
            kicker_id = %kicker_id,
            player_id = %player_id,
            player_email = %player_email,
            "Player kicked from game"
        );

        Ok(player_email)
    }

    /// Checks if a user is the creator of a game
    pub fn is_game_creator(&self, game_id: Uuid, user_id: Uuid) -> Result<bool, GameError> {
        let games = self.games.lock().unwrap();
        let game = games.get(&game_id).ok_or(GameError::GameNotFound)?;
        Ok(game.creator_id == user_id)
    }

    /// Finishes a game manually and returns the results
    ///
    /// # Security (Milestone 8)
    ///
    /// Only the game creator can manually finish a game.
    /// Games can also auto-finish when all players stand/bust.
    ///
    /// # Arguments
    ///
    /// * `game_id` - The game to finish
    /// * `user_id` - The user attempting to finish the game
    ///
    /// # Errors
    ///
    /// - `InsufficientPermissions` if user is not the creator
    /// - `GameNotFound` if game doesn't exist
    #[tracing::instrument(skip(self), fields(game_id, user_id))]
    pub fn finish_game(&self, game_id: Uuid, user_id: Uuid) -> Result<GameResult, GameError> {
        use blackjack_core::GamePermission;

        let mut games = self.games.lock().unwrap();
        let game = games.get_mut(&game_id).ok_or(GameError::GameNotFound)?;

        // Check if user has permission to finish game
        if !game.can_user_perform(user_id, GamePermission::FinishGame) {
            tracing::warn!(
                game_id = %game_id,
                user_id = %user_id,
                "Permission denied: user attempted to finish game"
            );
            return Err(GameError::InsufficientPermissions);
        }

        game.finish_game();
        let results = game.calculate_results();

        tracing::info!(
            game_id = %game_id,
            user_id = %user_id,
            winner = ?results.winner,
            highest_score = results.highest_score,
            "Game finished manually"
        );

        Ok(results)
    }

    /// Retrieves the results of a finished game
    ///
    /// Returns the game results including winner, tied players, and all player summaries.
    /// The game must be finished before calling this method.
    ///
    /// # Errors
    ///
    /// Returns `GameError::GameNotFound` if the game doesn't exist.
    #[tracing::instrument(skip(self), fields(game_id))]
    pub fn get_game_results(&self, game_id: Uuid) -> Result<GameResult, GameError> {
        let games = self.games.lock().unwrap();
        let game = games.get(&game_id).ok_or(GameError::GameNotFound)?;

        Ok(game.calculate_results())
    }

    /// Returns a reference to the service configuration
    pub fn config(&self) -> &ServiceConfig {
        &self.config
    }
}
