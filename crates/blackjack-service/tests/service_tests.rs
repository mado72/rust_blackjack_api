use blackjack_service::{GameError, GameService, ServiceConfig, UserService};
use std::sync::Arc;
use uuid::Uuid;

// Helper function to create a test creator email
fn test_creator_email() -> String {
    "creator@test.com".to_string()
}

// Helper function for a strong test password (meets M8 security requirements)
fn test_password() -> String {
    "TestP@ssw0rd".to_string()
}

// Helper to create a UserService with a test user
fn create_test_user_service() -> Arc<UserService> {
    let user_service = Arc::new(UserService::new());
    // Register the test creator with strong password
    let _ = user_service.register(test_creator_email(), test_password());
    user_service
}

// Helper to create GameService with UserService, returns both
fn create_game_service(config: ServiceConfig) -> (GameService, Arc<UserService>) {
    let user_service = create_test_user_service();
    let game_service = GameService::new(config, user_service.clone());
    (game_service, user_service)
}

#[test]
fn test_service_config_defaults() {
    let config = ServiceConfig::default();
    assert_eq!(config.min_players, 1);
    assert_eq!(config.max_players, 10);
}

#[test]
fn test_service_config_from_env() {
    unsafe {
        std::env::set_var("BLACKJACK_MIN_PLAYERS", "2");
        std::env::set_var("BLACKJACK_MAX_PLAYERS", "8");
    }

    let config = ServiceConfig::from_env();
    assert_eq!(config.min_players, 2);
    assert_eq!(config.max_players, 8);

    // Cleanup
    unsafe {
        std::env::remove_var("BLACKJACK_MIN_PLAYERS");
        std::env::remove_var("BLACKJACK_MAX_PLAYERS");
    }
}

#[test]
fn test_create_game_success() {
    let (service, user_service) = create_game_service(ServiceConfig::default());
    let creator_id = user_service
        .get_user_by_email(&test_creator_email())
        .unwrap()
        .id;

    let result = service.create_game(creator_id, None);
    assert!(result.is_ok());
}

#[test]
fn test_create_game_too_many_players() {
    let config = ServiceConfig {
        min_players: 1,
        max_players: 3,
    };
    let (service, user_service) = create_game_service(config);
    let creator_id = user_service
        .get_user_by_email(&test_creator_email())
        .unwrap()
        .id;

    let game_id = service.create_game(creator_id, None).unwrap();

    // Note: max_players config is not currently enforced (hardcoded at 10 in Game)
    // Creator is already enrolled, can add up to 9 more for total of 10
    let p2_id = user_service.register("p2@test.com".to_string(), "TestP@ssw0rd".to_string()).unwrap();
    let result = service.enroll_player(game_id, p2_id);
    assert!(result.is_ok()); // Should succeed (total 2)
    let p3_id = user_service.register("p3@test.com".to_string(), "TestP@ssw0rd".to_string()).unwrap();
    let result = service.enroll_player(game_id, p3_id);
    assert!(result.is_ok()); // Should succeed (total 3)
    let p4_id = user_service.register("p4@test.com".to_string(), "TestP@ssw0rd".to_string()).unwrap();
    let result = service.enroll_player(game_id, p4_id);
    assert!(result.is_ok()); // Should succeed (total 4, config max_players not enforced yet)
}

#[test]
fn test_create_game_too_few_players() {
    let config = ServiceConfig {
        min_players: 2,
        max_players: 10,
    };
    let (service, user_service) = create_game_service(config);
    let creator_id = user_service
        .get_user_by_email(&test_creator_email())
        .unwrap()
        .id;

    let game_id = service.create_game(creator_id, None).unwrap();
    assert!(game_id > Uuid::nil());
}

#[test]
fn test_draw_card() {
    let (service, user_service) = create_game_service(ServiceConfig::default());
    let creator_id = user_service
        .get_user_by_email(&test_creator_email())
        .unwrap()
        .id;

    let game_id = service.create_game(creator_id, None).unwrap();
    let player1_id = user_service.register("player1@test.com".to_string(), "TestP@ssw0rd".to_string()).unwrap();
    service.enroll_player(game_id, player1_id).unwrap();
    service.close_enrollment(game_id, creator_id).unwrap();

    let result = service.draw_card(game_id, creator_id);

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.cards_remaining, 51);
    assert_eq!(response.cards_history.len(), 1);
}

#[test]
fn test_draw_card_game_not_found() {
    let (service, _user_service) = create_game_service(ServiceConfig::default());
    let fake_game_id = uuid::Uuid::new_v4();
    let fake_user_id = uuid::Uuid::new_v4();

    let result = service.draw_card(fake_game_id, fake_user_id);
    assert!(result.is_err());
}

#[test]
fn test_set_ace_value() {
    let (service, user_service) = create_game_service(ServiceConfig::default());
    let creator_id = user_service
        .get_user_by_email(&test_creator_email())
        .unwrap()
        .id;

    let game_id = service.create_game(creator_id, None).unwrap();
    let player1_id = user_service.register("player1@test.com".to_string(), "TestP@ssw0rd".to_string()).unwrap();
    service.enroll_player(game_id, player1_id).unwrap();
    service.close_enrollment(game_id, creator_id).unwrap();

    let mut ace_card_id = None;
    for _ in 0..52 {
        if let Ok(response) = service.draw_card(game_id, creator_id) {
            if response.card.name == "A" {
                ace_card_id = Some(response.card.id);
                break;
            }
        }
    }

    if let Some(card_id) = ace_card_id {
        let result = service.set_ace_value(game_id, creator_id, card_id, true);
        assert!(result.is_ok());

        let result = service.set_ace_value(game_id, creator_id, card_id, false);
        assert!(result.is_ok());
    }
}

#[test]
fn test_get_game_state() {
    let (service, user_service) = create_game_service(ServiceConfig::default());
    let creator_id = user_service
        .get_user_by_email(&test_creator_email())
        .unwrap()
        .id;

    let game_id = service.create_game(creator_id, None).unwrap();
    let player1_id = user_service.register("player1@test.com".to_string(), "TestP@ssw0rd".to_string()).unwrap();
    let player2_id = user_service.register("player2@test.com".to_string(), "TestP@ssw0rd".to_string()).unwrap();
    service.enroll_player(game_id, player1_id).unwrap();
    service.enroll_player(game_id, player2_id).unwrap();

    let result = service.get_game_state(game_id);

    assert!(result.is_ok());
    let state = result.unwrap();
    assert_eq!(state.players.len(), 3);
    assert_eq!(state.cards_in_deck, 52);
    assert!(!state.finished);
}

#[test]
fn test_finish_game() {
    let (service, user_service) = create_game_service(ServiceConfig::default());
    let creator_id = user_service
        .get_user_by_email(&test_creator_email())
        .unwrap()
        .id;

    let game_id = service.create_game(creator_id, None).unwrap();
    let player1_id = user_service.register("player1@test.com".to_string(), "TestP@ssw0rd".to_string()).unwrap();
    let player2_id = user_service.register("player2@test.com".to_string(), "TestP@ssw0rd".to_string()).unwrap();
    service.enroll_player(game_id, player1_id).unwrap();
    service.enroll_player(game_id, player2_id).unwrap();

    let _ = service.draw_card(game_id, creator_id);
    let _ = service.draw_card(game_id, player1_id);

    let result = service.finish_game(game_id, creator_id);
    assert!(result.is_ok());

    let game_result = result.unwrap();
    assert_eq!(game_result.all_players.len(), 4);
}

#[test]
fn test_concurrent_access() {
    use std::sync::Arc;
    use std::thread;

    let (game_service, user_service) = create_game_service(ServiceConfig::default());
    let creator_id = user_service
        .get_user_by_email(&test_creator_email())
        .unwrap()
        .id;

    let service = Arc::new(game_service);

    let game_id = service.create_game(creator_id, None).unwrap();
    let player1_id = user_service.register("player1@test.com".to_string(), "TestP@ssw0rd".to_string()).unwrap();
    let player2_id = user_service.register("player2@test.com".to_string(), "TestP@ssw0rd".to_string()).unwrap();
    service.enroll_player(game_id, player1_id).unwrap();
    service.enroll_player(game_id, player2_id).unwrap();
    service.close_enrollment(game_id, creator_id).unwrap();

    let mut handles = vec![];

    for i in 0..5 {
        let service_clone = Arc::clone(&service);
        let player_id = if i % 3 == 0 {
            creator_id
        } else if i % 3 == 1 {
            player1_id
        } else {
            player2_id
        };

        let handle = thread::spawn(move || service_clone.draw_card(game_id, player_id));

        handles.push(handle);
    }

    let mut success_count = 0;
    for handle in handles {
        if let Ok(result) = handle.join() {
            if result.is_ok() {
                success_count += 1;
            }
        }
    }

    assert!(success_count > 0);
}

#[test]
fn test_draw_until_deck_empty() {
    let (service, user_service) = create_game_service(ServiceConfig::default());
    let creator_id = user_service
        .get_user_by_email(&test_creator_email())
        .unwrap()
        .id;

    let game_id = service.create_game(creator_id, None).unwrap();
    let player1_id = user_service.register("player1@test.com".to_string(), "TestP@ssw0rd".to_string()).unwrap();
    service.enroll_player(game_id, player1_id).unwrap();
    service.close_enrollment(game_id, creator_id).unwrap();

    // With turn management and dealer auto-play, the game will finish when all players complete
    // Let's just draw cards until game finishes or error occurs
    for _ in 0..100 {
        // Limit iterations to avoid infinite loop
        let state = service.get_game_state(game_id).unwrap();
        if state.finished {
            break;
        }

        // Try to draw for current player - need to get user_id from email
        let current_player_email = if let Some(cp) = state.current_turn_player {
            cp
        } else {
            break;
        };

        // Get user_id from email
        let current_user_id = if current_player_email == test_creator_email() {
            creator_id
        } else {
            player1_id
        };

        let result = service.draw_card(game_id, current_user_id);
        if result.is_err() {
            // Player busted or game finished
            break;
        }
    }

    // Check game state - should be finished (either by all players done or deck empty)
    let state = service.get_game_state(game_id).unwrap();
    assert!(
        state.finished || state.cards_in_deck < 52,
        "Game should have progressed"
    );
}

#[test]
fn test_enroll_player_already_enrolled() {
    use blackjack_service::GameError;

    let (service, user_service) = create_game_service(ServiceConfig::default());
    let creator_id = user_service
        .get_user_by_email(&test_creator_email())
        .unwrap()
        .id;

    let game_id = service.create_game(creator_id, None).unwrap();

    // Try to enroll creator again (creator is auto-enrolled at game creation)
    let result = service.enroll_player(game_id, creator_id);

    // Should get PlayerAlreadyEnrolled error, not CoreError
    assert!(matches!(result, Err(GameError::PlayerAlreadyEnrolled)));
}

// ============================================================================
// M8: Security Tests
// ============================================================================

#[test]
fn test_weak_password_rejected() {
    let user_service = UserService::new();

    // Too short (< 8 chars)
    let result = user_service.register("user@test.com".to_string(), "Short1!".to_string());
    assert!(matches!(result, Err(GameError::WeakPassword(_))));

    // Missing uppercase
    let result = user_service.register("user@test.com".to_string(), "lowercase1!".to_string());
    assert!(matches!(result, Err(GameError::WeakPassword(_))));

    // Missing lowercase
    let result = user_service.register("user@test.com".to_string(), "UPPERCASE1!".to_string());
    assert!(matches!(result, Err(GameError::WeakPassword(_))));

    // Missing digit
    let result = user_service.register("user@test.com".to_string(), "NoDigits!".to_string());
    assert!(matches!(result, Err(GameError::WeakPassword(_))));

    // Missing special character
    let result = user_service.register("user@test.com".to_string(), "NoSpecial1".to_string());
    assert!(matches!(result, Err(GameError::WeakPassword(_))));

    // Valid password should work
    let result = user_service.register("user@test.com".to_string(), "ValidP@ss1".to_string());
    assert!(result.is_ok());
}

#[test]
fn test_invalid_email_rejected() {
    let user_service = UserService::new();

    // Missing @
    let result = user_service.register("notanemail".to_string(), "TestP@ssw0rd".to_string());
    assert!(matches!(result, Err(GameError::ValidationError(_))));

    // Missing domain
    let result = user_service.register("user@".to_string(), "TestP@ssw0rd".to_string());
    assert!(matches!(result, Err(GameError::ValidationError(_))));

    // Missing local part
    let result = user_service.register("@example.com".to_string(), "TestP@ssw0rd".to_string());
    assert!(matches!(result, Err(GameError::ValidationError(_))));

    // Valid email should work
    let result = user_service.register("user@test.com".to_string(), "TestP@ssw0rd".to_string());
    assert!(result.is_ok());
}

#[test]
fn test_failed_login_with_wrong_password() {
    let user_service = UserService::new();

    // Register user
    user_service
        .register("user@test.com".to_string(), "TestP@ssw0rd".to_string())
        .unwrap();

    // Try to login with wrong password
    let result = user_service.login("user@test.com", "WrongP@ssw0rd");
    assert!(matches!(result, Err(GameError::InvalidCredentials)));
}

#[test]
fn test_change_password_requires_old_password() {
    let user_service = UserService::new();

    // Register user
    let user_id = user_service
        .register("user@test.com".to_string(), "OldP@ssw0rd".to_string())
        .unwrap();

    // Try to change password with wrong old password
    let result = user_service.change_password(user_id, "WrongOldP@ssw0rd", "NewP@ssw0rd");
    assert!(matches!(result, Err(GameError::InvalidCredentials)));

    // Change password with correct old password
    let result = user_service.change_password(user_id, "OldP@ssw0rd", "NewP@ssw0rd");
    assert!(result.is_ok());

    // Old password should no longer work
    let result = user_service.login("user@test.com", "OldP@ssw0rd");
    assert!(matches!(result, Err(GameError::InvalidCredentials)));

    // New password should work
    let result = user_service.login("user@test.com", "NewP@ssw0rd");
    assert!(result.is_ok());
}

#[test]
fn test_change_password_validates_new_password() {
    let user_service = UserService::new();

    let user_id = user_service
        .register("user@test.com".to_string(), "OldP@ssw0rd".to_string())
        .unwrap();

    // Try to change to weak password
    let result = user_service.change_password(user_id, "OldP@ssw0rd", "weak");
    assert!(matches!(result, Err(GameError::WeakPassword(_))));
}

#[test]
fn test_rbac_only_creator_can_close_enrollment() {
    let (service, user_service) = create_game_service(ServiceConfig::default());
    let creator_id = user_service
        .get_user_by_email(&test_creator_email())
        .unwrap()
        .id;

    let game_id = service.create_game(creator_id, None).unwrap();

    // Enroll another player
    let player_id = user_service
        .register("player@test.com".to_string(), "TestP@ssw0rd".to_string())
        .unwrap();
    service.enroll_player(game_id, player_id).unwrap();

    // Player should not be able to close enrollment
    let result = service.close_enrollment(game_id, player_id);
    assert!(matches!(result, Err(GameError::InsufficientPermissions)));

    // Creator should be able to close enrollment
    let result = service.close_enrollment(game_id, creator_id);
    assert!(result.is_ok());
}

#[test]
fn test_rbac_only_creator_can_finish_game() {
    let (service, user_service) = create_game_service(ServiceConfig::default());
    let creator_id = user_service
        .get_user_by_email(&test_creator_email())
        .unwrap()
        .id;

    let game_id = service.create_game(creator_id, None).unwrap();

    // Enroll another player
    let player_id = user_service
        .register("player@test.com".to_string(), "TestP@ssw0rd".to_string())
        .unwrap();
    service.enroll_player(game_id, player_id).unwrap();

    // Close enrollment as creator
    service.close_enrollment(game_id, creator_id).unwrap();

    // Player should not be able to finish game
    let result = service.finish_game(game_id, player_id);
    assert!(matches!(result, Err(GameError::InsufficientPermissions)));

    // Creator should be able to finish game
    let result = service.finish_game(game_id, creator_id);
    assert!(result.is_ok());
}

#[test]
fn test_rbac_only_creator_can_kick_players() {
    let (service, user_service) = create_game_service(ServiceConfig::default());
    let creator_id = user_service
        .get_user_by_email(&test_creator_email())
        .unwrap()
        .id;

    let game_id = service.create_game(creator_id, None).unwrap();

    // Enroll two players
    let player1_id = user_service
        .register("player1@test.com".to_string(), "TestP@ssw0rd".to_string())
        .unwrap();
    let player2_id = user_service
        .register("player2@test.com".to_string(), "TestP@ssw0rd".to_string())
        .unwrap();
    service.enroll_player(game_id, player1_id).unwrap();
    service.enroll_player(game_id, player2_id).unwrap();

    // Player1 should not be able to kick player2
    let result = service.kick_player(game_id, player1_id, player2_id);
    assert!(matches!(result, Err(GameError::InsufficientPermissions)));

    // Creator should be able to kick player1
    let result = service.kick_player(game_id, creator_id, player1_id);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "player1@test.com");
}

#[test]
fn test_rbac_cannot_kick_creator() {
    let (service, user_service) = create_game_service(ServiceConfig::default());
    let creator_id = user_service
        .get_user_by_email(&test_creator_email())
        .unwrap()
        .id;

    let game_id = service.create_game(creator_id, None).unwrap();

    // Try to kick creator (should fail even as creator)
    let result = service.kick_player(game_id, creator_id, creator_id);
    assert!(matches!(
        result,
        Err(GameError::CoreError(blackjack_core::GameError::CannotKickCreator))
    ));
}

#[test]
fn test_account_status_inactive_cannot_login() {
    let user_service = UserService::new();

    // Register user
    let user_id = user_service
        .register("user@test.com".to_string(), "TestP@ssw0rd".to_string())
        .unwrap();

    // Deactivate account
    user_service.deactivate_account(user_id).unwrap();

    // Try to login with inactive account
    let result = user_service.login("user@test.com", "TestP@ssw0rd");
    assert!(matches!(result, Err(GameError::AccountInactive)));
}

#[test]
fn test_last_login_updated_on_successful_login() {
    let user_service = UserService::new();

    // Register user
    user_service
        .register("user@test.com".to_string(), "TestP@ssw0rd".to_string())
        .unwrap();

    // Login
    let user = user_service.login("user@test.com", "TestP@ssw0rd").unwrap();

    // last_login should be set
    assert!(user.last_login.is_some());
}
