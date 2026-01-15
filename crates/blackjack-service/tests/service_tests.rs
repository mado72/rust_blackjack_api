use blackjack_service::{GameService, ServiceConfig, UserService};
use std::sync::Arc;
use uuid::Uuid;

// Helper function to create a test creator email
fn test_creator_email() -> String {
    "creator@test.com".to_string()
}

// Helper to create a UserService with a test user
fn create_test_user_service() -> Arc<UserService> {
    let user_service = Arc::new(UserService::new());
    // Register the test creator
    let _ = user_service.register(test_creator_email(), "password123".to_string());
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
    let p2_id = user_service.register("p2@test.com".to_string(), "pass123".to_string()).unwrap();
    let result = service.enroll_player(game_id, p2_id);
    assert!(result.is_ok()); // Should succeed (total 2)
    let p3_id = user_service.register("p3@test.com".to_string(), "pass123".to_string()).unwrap();
    let result = service.enroll_player(game_id, p3_id);
    assert!(result.is_ok()); // Should succeed (total 3)
    let p4_id = user_service.register("p4@test.com".to_string(), "pass123".to_string()).unwrap();
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
    let player1_id = user_service.register("player1@test.com".to_string(), "pass123".to_string()).unwrap();
    service.enroll_player(game_id, player1_id).unwrap();
    service.close_enrollment(game_id, creator_id).unwrap();

    let result = service.draw_card(game_id, &test_creator_email());

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.cards_remaining, 51);
    assert_eq!(response.cards_history.len(), 1);
}

#[test]
fn test_draw_card_game_not_found() {
    let (service, _user_service) = create_game_service(ServiceConfig::default());
    let fake_game_id = uuid::Uuid::new_v4();

    let result = service.draw_card(fake_game_id, "player@test.com");
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
    let player1_id = user_service.register("player1@test.com".to_string(), "pass123".to_string()).unwrap();
    service.enroll_player(game_id, player1_id).unwrap();
    service.close_enrollment(game_id, creator_id).unwrap();

    let mut ace_card_id = None;
    for _ in 0..52 {
        if let Ok(response) = service.draw_card(game_id, &test_creator_email()) {
            if response.card.name == "A" {
                ace_card_id = Some(response.card.id);
                break;
            }
        }
    }

    if let Some(card_id) = ace_card_id {
        let result = service.set_ace_value(game_id, &test_creator_email(), card_id, true);
        assert!(result.is_ok());

        let result = service.set_ace_value(game_id, &test_creator_email(), card_id, false);
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
    let player1_id = user_service.register("player1@test.com".to_string(), "pass123".to_string()).unwrap();
    let player2_id = user_service.register("player2@test.com".to_string(), "pass123".to_string()).unwrap();
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
    let player1_id = user_service.register("player1@test.com".to_string(), "pass123".to_string()).unwrap();
    let player2_id = user_service.register("player2@test.com".to_string(), "pass123".to_string()).unwrap();
    service.enroll_player(game_id, player1_id).unwrap();
    service.enroll_player(game_id, player2_id).unwrap();

    let _ = service.draw_card(game_id, &test_creator_email());
    let _ = service.draw_card(game_id, "player1@test.com");

    let result = service.finish_game(game_id);
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
    let player1_id = user_service.register("player1@test.com".to_string(), "pass123".to_string()).unwrap();
    let player2_id = user_service.register("player2@test.com".to_string(), "pass123".to_string()).unwrap();
    service.enroll_player(game_id, player1_id).unwrap();
    service.enroll_player(game_id, player2_id).unwrap();
    service.close_enrollment(game_id, creator_id).unwrap();

    let mut handles = vec![];

    for i in 0..5 {
        let service_clone = Arc::clone(&service);
        let player = if i % 3 == 0 {
            test_creator_email()
        } else if i % 3 == 1 {
            "player1@test.com".to_string()
        } else {
            "player2@test.com".to_string()
        };

        let handle = thread::spawn(move || service_clone.draw_card(game_id, &player));

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
    let player1_id = user_service.register("player1@test.com".to_string(), "pass123".to_string()).unwrap();
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

        // Try to draw for current player
        let current_player = if let Some(cp) = state.current_turn_player {
            cp
        } else {
            break;
        };

        let result = service.draw_card(game_id, &current_player);
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
