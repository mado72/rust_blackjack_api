use blackjack_service::{GameService, ServiceConfig};

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
    let service = GameService::new_default();
    let emails = vec!["player1@test.com".to_string(), "player2@test.com".to_string()];

    let result = service.create_game(emails);
    assert!(result.is_ok());
}

#[test]
fn test_create_game_too_many_players() {
    let config = ServiceConfig {
        min_players: 1,
        max_players: 3,
    };
    let service = GameService::new(config);

    let emails = vec![
        "p1@test.com".to_string(),
        "p2@test.com".to_string(),
        "p3@test.com".to_string(),
        "p4@test.com".to_string(),
    ];

    let result = service.create_game(emails);
    assert!(result.is_err());
}

#[test]
fn test_create_game_too_few_players() {
    let config = ServiceConfig {
        min_players: 2,
        max_players: 10,
    };
    let service = GameService::new(config);

    let emails = vec!["p1@test.com".to_string()];

    let result = service.create_game(emails);
    assert!(result.is_err());
}

#[test]
fn test_draw_card() {
    let service = GameService::new_default();
    let emails = vec!["player1@test.com".to_string()];

    let game_id = service.create_game(emails.clone()).unwrap();
    let result = service.draw_card(game_id, "player1@test.com");

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.cards_remaining, 51); // 52 - 1
    assert_eq!(response.cards_history.len(), 1);
}

#[test]
fn test_draw_card_game_not_found() {
    let service = GameService::new_default();
    let fake_game_id = uuid::Uuid::new_v4();

    let result = service.draw_card(fake_game_id, "player@test.com");
    assert!(result.is_err());
}

#[test]
fn test_set_ace_value() {
    let service = GameService::new_default();
    let emails = vec!["player1@test.com".to_string()];

    let game_id = service.create_game(emails).unwrap();

    // Draw cards until we get an Ace
    let mut ace_card_id = None;
    for _ in 0..52 {
        if let Ok(response) = service.draw_card(game_id, "player1@test.com") {
            if response.card.name == "A" {
                ace_card_id = Some(response.card.id);
                break;
            }
        }
    }

    if let Some(card_id) = ace_card_id {
        let result = service.set_ace_value(game_id, "player1@test.com", card_id, true);
        assert!(result.is_ok());

        // Change it again to false
        let result = service.set_ace_value(game_id, "player1@test.com", card_id, false);
        assert!(result.is_ok());
    }
}

#[test]
fn test_get_game_state() {
    let service = GameService::new_default();
    let emails = vec!["player1@test.com".to_string(), "player2@test.com".to_string()];

    let game_id = service.create_game(emails).unwrap();
    let result = service.get_game_state(game_id);

    assert!(result.is_ok());
    let state = result.unwrap();
    assert_eq!(state.players.len(), 2);
    assert_eq!(state.cards_in_deck, 52);
    assert!(!state.finished);
}

#[test]
fn test_finish_game() {
    let service = GameService::new_default();
    let emails = vec!["player1@test.com".to_string(), "player2@test.com".to_string()];

    let game_id = service.create_game(emails).unwrap();

    // Draw some cards
    let _ = service.draw_card(game_id, "player1@test.com");
    let _ = service.draw_card(game_id, "player2@test.com");

    let result = service.finish_game(game_id);
    assert!(result.is_ok());

    let game_result = result.unwrap();
    assert_eq!(game_result.all_players.len(), 2);
}

#[test]
fn test_concurrent_access() {
    use std::sync::Arc;
    use std::thread;

    let service = Arc::new(GameService::new_default());
    let emails = vec!["player1@test.com".to_string(), "player2@test.com".to_string()];

    let game_id = service.create_game(emails).unwrap();

    let mut handles = vec![];

    // Spawn multiple threads trying to draw cards simultaneously
    for i in 0..5 {
        let service_clone = Arc::clone(&service);
        let player = if i % 2 == 0 {
            "player1@test.com"
        } else {
            "player2@test.com"
        };

        let handle = thread::spawn(move || {
            service_clone.draw_card(game_id, player)
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    let mut success_count = 0;
    for handle in handles {
        if let Ok(result) = handle.join() {
            if result.is_ok() {
                success_count += 1;
            }
        }
    }

    // At least some draws should succeed
    assert!(success_count > 0);
}

#[test]
fn test_draw_until_deck_empty() {
    let service = GameService::new_default();
    let emails = vec!["player1@test.com".to_string()];

    let game_id = service.create_game(emails).unwrap();

    // Draw all 52 cards
    for _ in 0..52 {
        let result = service.draw_card(game_id, "player1@test.com");
        if result.is_err() {
            // Player might bust before deck is empty
            break;
        }
    }

    // Check game state
    let state = service.get_game_state(game_id).unwrap();
    // Either deck is empty or player busted
    assert!(state.cards_in_deck == 0 || state.players.get("player1@test.com").unwrap().busted);
}
