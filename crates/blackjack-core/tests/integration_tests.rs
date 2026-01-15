use blackjack_core::{Game, GameError, PlayerState};
use uuid::Uuid;

// Helper function to create a test creator_id
fn test_creator_id() -> Uuid {
    Uuid::new_v4()
}

// Helper function to create a test creator email
fn test_creator_email() -> String {
    "creator@test.com".to_string()
}

// Helper function to create a game with default enrollment timeout
fn test_game(emails: Vec<&str>) -> Result<Game, GameError> {
    let mut game = Game::new(test_creator_id(), test_creator_email(), 300)?;

    // Enroll additional players (creator is already enrolled)
    for email in emails {
        if email != "creator@test.com" {
            game.add_player(email.to_string())?;
        }
    }

    // Close enrollment to allow gameplay
    game.close_enrollment()?;
    Ok(game)
}

#[test]
fn test_deck_has_52_cards() {
    let game = test_game(vec!["player1@test.com"]).unwrap();
    assert_eq!(
        game.available_cards.len(),
        52,
        "Deck should have exactly 52 cards"
    );
}

#[test]
fn test_four_cards_of_each_type() {
    let game = test_game(vec!["player1@test.com"]).unwrap();

    // Count cards by name
    let card_types = [
        "A", "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K",
    ];
    for card_type in card_types.iter() {
        let count = game
            .available_cards
            .iter()
            .filter(|c| c.name == *card_type)
            .count();
        assert_eq!(
            count, 4,
            "Should have exactly 4 cards of type {}",
            card_type
        );
    }
}

#[test]
fn test_cards_have_correct_suits() {
    let game = test_game(vec!["player1@test.com"]).unwrap();

    let suits = ["Hearts", "Diamonds", "Clubs", "Spades"];
    for suit in suits.iter() {
        let count = game
            .available_cards
            .iter()
            .filter(|c| c.suit == *suit)
            .count();
        assert_eq!(count, 13, "Should have exactly 13 cards of suit {}", suit);
    }
}

#[test]
fn test_deck_exhaustion() {
    let mut game = test_game(vec!["player1@test.com"]).unwrap();

    // Draw cards until player busts or game finishes
    let mut cards_drawn = 0;
    loop {
        // Try the current player
        let current_player = game.get_current_player().unwrap_or("").to_string();
        if current_player.is_empty() {
            break;
        }

        let result = game.draw_card(&current_player);
        match result {
            Ok(_) => cards_drawn += 1,
            Err(GameError::PlayerAlreadyBusted) => {
                // Player busted, which is expected
                break;
            }
            Err(GameError::DeckEmpty) => {
                // Deck is empty
                break;
            }
            Err(GameError::GameAlreadyFinished) => {
                // Game finished (auto-finish after all players done)
                break;
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }

        if cards_drawn >= 52 {
            break;
        }
    }

    // Verify we can create a game and test DeckEmpty by manually emptying deck
    let mut game2 = test_game(vec!["player1@test.com"]).unwrap();

    // Manually empty the deck
    game2.available_cards.clear();

    // Now try to draw from empty deck
    let result = game2.draw_card("creator@test.com");
    assert_eq!(
        result,
        Err(GameError::DeckEmpty),
        "Should return DeckEmpty error"
    );
}

#[test]
fn test_ace_value_can_be_changed_multiple_times() {
    let mut game = test_game(vec!["player1@test.com"]).unwrap();

    // Find an Ace in the deck and give it to the player
    let ace_index = game
        .available_cards
        .iter()
        .position(|c| c.name == "A")
        .expect("Should have an Ace in deck");
    let ace = game.available_cards.remove(ace_index);
    let ace_id = ace.id;

    game.players
        .get_mut("player1@test.com")
        .unwrap()
        .add_card(ace);

    let player = game.players.get("player1@test.com").unwrap();
    let initial_points = player.points;

    // Change Ace to 11
    game.set_ace_value("player1@test.com", ace_id, true)
        .unwrap();
    let player = game.players.get("player1@test.com").unwrap();
    assert_eq!(
        player.points,
        initial_points + 10,
        "Points should increase by 10 when Ace is 11"
    );

    // Change back to 1
    game.set_ace_value("player1@test.com", ace_id, false)
        .unwrap();
    let player = game.players.get("player1@test.com").unwrap();
    assert_eq!(
        player.points, initial_points,
        "Points should return to initial when Ace is 1"
    );

    // Change to 11 again
    game.set_ace_value("player1@test.com", ace_id, true)
        .unwrap();
    let player = game.players.get("player1@test.com").unwrap();
    assert_eq!(
        player.points,
        initial_points + 10,
        "Can change Ace value multiple times"
    );
}

#[test]
fn test_game_finished_prevents_draw() {
    let mut game = test_game(vec!["player1@test.com"]).unwrap();

    game.finish_game();

    let result = game.draw_card("player1@test.com");
    assert_eq!(
        result,
        Err(GameError::GameAlreadyFinished),
        "Cannot draw after game finished"
    );
}

#[test]
fn test_game_finished_prevents_ace_change() {
    let mut game = test_game(vec!["player1@test.com"]).unwrap();

    // Find an Ace and give it to the player
    let ace_index = game
        .available_cards
        .iter()
        .position(|c| c.name == "A")
        .expect("Should have an Ace in deck");
    let ace = game.available_cards.remove(ace_index);
    let ace_id = ace.id;

    game.players
        .get_mut("player1@test.com")
        .unwrap()
        .add_card(ace);

    game.finish_game();

    let result = game.set_ace_value("player1@test.com", ace_id, true);
    assert_eq!(
        result,
        Err(GameError::GameAlreadyFinished),
        "Cannot change Ace after game finished"
    );
}

#[test]
fn test_json_serialization() {
    let game = test_game(vec!["player1@test.com"]).unwrap();

    // Serialize to JSON
    let json = serde_json::to_string(&game).expect("Should serialize to JSON");

    // Deserialize back
    let deserialized: Game = serde_json::from_str(&json).expect("Should deserialize from JSON");

    assert_eq!(game.id, deserialized.id);
    assert_eq!(game.players.len(), deserialized.players.len());
    assert_eq!(
        game.available_cards.len(),
        deserialized.available_cards.len()
    );
    assert_eq!(game.finished, deserialized.finished);
}

#[test]
fn test_calculate_results_single_winner() {
    let mut game = test_game(vec![
        "player1@test.com",
        "player2@test.com",
        "player3@test.com",
    ])
    .unwrap();

    // Manually set points (simulating game play)
    game.players.get_mut("player1@test.com").unwrap().points = 15;
    game.players.get_mut("player2@test.com").unwrap().points = 20;
    game.players.get_mut("player3@test.com").unwrap().points = 18;

    let result = game.calculate_results();

    assert_eq!(result.winner, Some("player2@test.com".to_string()));
    assert_eq!(result.highest_score, 20);
    assert!(result.tied_players.is_empty());
}

#[test]
fn test_calculate_results_tie() {
    let mut game = test_game(vec![
        "player1@test.com",
        "player2@test.com",
        "player3@test.com",
    ])
    .unwrap();

    game.players.get_mut("player1@test.com").unwrap().points = 19;
    game.players.get_mut("player2@test.com").unwrap().points = 15;
    game.players.get_mut("player3@test.com").unwrap().points = 19;

    let result = game.calculate_results();

    assert_eq!(result.winner, None);
    assert_eq!(result.highest_score, 19);
    assert_eq!(result.tied_players.len(), 2);
    assert!(
        result
            .tied_players
            .contains(&"player1@test.com".to_string())
    );
    assert!(
        result
            .tied_players
            .contains(&"player3@test.com".to_string())
    );
}

#[test]
fn test_calculate_results_all_bust() {
    let mut game = test_game(vec!["player1@test.com", "player2@test.com"]).unwrap();

    game.players.get_mut("player1@test.com").unwrap().points = 22;
    game.players.get_mut("player1@test.com").unwrap().busted = true;
    game.players.get_mut("player2@test.com").unwrap().points = 25;
    game.players.get_mut("player2@test.com").unwrap().busted = true;

    let result = game.calculate_results();

    assert_eq!(result.winner, None);
    assert_eq!(result.highest_score, 0);
    assert!(result.tied_players.is_empty());
}

#[test]
fn test_calculate_results_perfect_21() {
    let mut game = test_game(vec![
        "player1@test.com",
        "player2@test.com",
        "player3@test.com",
    ])
    .unwrap();

    game.players.get_mut("player1@test.com").unwrap().points = 20;
    game.players.get_mut("player2@test.com").unwrap().points = 21;
    game.players.get_mut("player3@test.com").unwrap().points = 19;

    let result = game.calculate_results();

    assert_eq!(result.winner, Some("player2@test.com".to_string()));
    assert_eq!(result.highest_score, 21);
}

#[test]
fn test_invalid_player_count_zero() {
    // In the new M7 model, games start with creator already enrolled
    let result = Game::new(test_creator_id(), test_creator_email(), 300);
    assert!(
        result.is_ok(),
        "Game should be created with creator enrolled"
    );

    let game = result.unwrap();
    assert_eq!(game.players.len(), 1, "Game should have 1 player (creator)");
}

#[test]
fn test_invalid_player_count_too_many() {
    // In new M7 model, creator is auto-enrolled, so we can add 9 more players
    let mut game = Game::new(test_creator_id(), test_creator_email(), 300).unwrap();

    // Try to add 10 more players (total would be 11 with creator)
    for i in 1..=10 {
        let email = format!("player{}@test.com", i);
        if i <= 9 {
            assert!(
                game.add_player(email).is_ok(),
                "Should allow up to 10 total players (9 + creator)"
            );
        } else {
            assert!(
                game.add_player(email).is_err(),
                "Should reject more than 10 players"
            );
        }
    }
}

#[test]
fn test_invalid_email_empty() {
    // Test creating game with empty creator email
    let result = Game::new(test_creator_id(), "".to_string(), 300);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), GameError::InvalidEmail));
}

#[test]
fn test_invalid_email_duplicate() {
    // Creator is auto-enrolled, so trying to add creator again should fail
    let mut game = Game::new(test_creator_id(), test_creator_email(), 300).unwrap();

    let result = game.add_player(test_creator_email());
    assert!(result.is_err(), "Duplicate email (creator) should fail");
}

#[test]
fn test_player_not_in_game() {
    let mut game = test_game(vec!["player1@test.com"]).unwrap();

    let result = game.draw_card("nonexistent@test.com");
    // With turn management, it will return NotPlayerTurn before checking if player exists
    assert_eq!(result, Err(GameError::NotPlayerTurn));
}

#[test]
fn test_busted_player_cannot_draw() {
    let mut game = test_game(vec!["player1@test.com"]).unwrap();

    // Manually set creator (current player) as busted
    game.players.get_mut("creator@test.com").unwrap().busted = true;

    let result = game.draw_card("creator@test.com");
    assert_eq!(result, Err(GameError::PlayerAlreadyBusted));
}

#[test]
fn test_valid_player_range() {
    // In new M8 model, creating a game automatically enrolls the creator (1 player)
    for _count in 0..=10 {
        let result = Game::new(test_creator_id(), format!("user{}@test.com", _count), 300);
        assert!(
            result.is_ok(),
            "Should accept game creation with valid email"
        );

        if let Ok(game) = result {
            assert_eq!(
                game.players.len(),
                1,
                "Game should start with 1 player (creator)"
            );
        }
    }
}

// =====================================
// PHASE 2 TESTS - Turn Management & Stand
// =====================================

#[test]
fn test_player_state_initial() {
    use blackjack_core::PlayerState;
    let game = test_game(vec!["player1@test.com"]).unwrap();

    // Check both creator and enrolled player
    let creator = game.players.get("creator@test.com").unwrap();
    assert_eq!(
        creator.state,
        PlayerState::Active,
        "Creator should be Active"
    );

    let player = game.players.get("player1@test.com").unwrap();
    assert_eq!(
        player.state,
        PlayerState::Active,
        "New players should be Active"
    );
}

#[test]
fn test_get_current_player() {
    let game = test_game(vec![
        "player1@test.com",
        "player2@test.com",
        "player3@test.com",
    ])
    .unwrap();

    // Creator is enrolled first, so should have first turn
    let current = game.get_current_player();
    assert_eq!(
        current,
        Some("creator@test.com"),
        "Creator should have first turn"
    );
}

#[test]
fn test_advance_turn() {
    let mut game = test_game(vec![
        "player1@test.com",
        "player2@test.com",
        "player3@test.com",
    ])
    .unwrap();

    assert_eq!(game.get_current_player(), Some("creator@test.com"));

    game.advance_turn();
    assert_eq!(game.get_current_player(), Some("player1@test.com"));

    game.advance_turn();
    assert_eq!(game.get_current_player(), Some("player2@test.com"));

    game.advance_turn();
    assert_eq!(game.get_current_player(), Some("player3@test.com"));

    game.advance_turn();
    assert_eq!(
        game.get_current_player(),
        Some("creator@test.com"),
        "Should wrap around to creator"
    );
}

#[test]
fn test_advance_turn_skips_standing_players() {
    use blackjack_core::PlayerState;
    let mut game = test_game(vec![
        "player1@test.com",
        "player2@test.com",
        "player3@test.com",
    ])
    .unwrap();

    // Mark player1 as standing (second in turn order after creator)
    game.players.get_mut("player1@test.com").unwrap().state = PlayerState::Standing;

    assert_eq!(game.get_current_player(), Some("creator@test.com"));

    game.advance_turn();
    // Should skip player1 and go to player2
    assert_eq!(game.get_current_player(), Some("player2@test.com"));
}

#[test]
fn test_advance_turn_skips_busted_players() {
    use blackjack_core::PlayerState;
    let mut game = test_game(vec![
        "player1@test.com",
        "player2@test.com",
        "player3@test.com",
    ])
    .unwrap();

    // Mark player1 as busted (second in turn order after creator)
    game.players.get_mut("player1@test.com").unwrap().state = PlayerState::Busted;

    assert_eq!(game.get_current_player(), Some("creator@test.com"));

    game.advance_turn();
    // Should skip player1 and go to player2
    assert_eq!(game.get_current_player(), Some("player2@test.com"));
}

#[test]
fn test_stand_marks_player_as_standing() {
    use blackjack_core::PlayerState;
    let mut game = test_game(vec!["player1@test.com", "player2@test.com"]).unwrap();

    assert_eq!(game.get_current_player(), Some("creator@test.com"));

    game.stand("creator@test.com").unwrap();

    let player = game.players.get("creator@test.com").unwrap();
    assert_eq!(player.state, PlayerState::Standing);
}

#[test]
fn test_stand_advances_turn() {
    let mut game = test_game(vec!["player1@test.com", "player2@test.com"]).unwrap();

    assert_eq!(game.get_current_player(), Some("creator@test.com"));

    game.stand("creator@test.com").unwrap();

    assert_eq!(game.get_current_player(), Some("player1@test.com"));
}

#[test]
fn test_stand_not_your_turn() {
    let mut game = test_game(vec!["player1@test.com", "player2@test.com"]).unwrap();

    assert_eq!(game.get_current_player(), Some("creator@test.com"));

    let result = game.stand("player1@test.com");
    assert_eq!(result, Err(GameError::NotPlayerTurn));
}

#[test]
fn test_stand_auto_finishes_game() {
    let mut game = test_game(vec!["player1@test.com", "player2@test.com"]).unwrap();

    assert!(!game.finished);

    game.stand("creator@test.com").unwrap();
    assert!(
        !game.finished,
        "Game should not finish with players remaining"
    );

    game.stand("player1@test.com").unwrap();
    assert!(
        !game.finished,
        "Game should not finish with one player remaining"
    );

    game.stand("player2@test.com").unwrap();
    assert!(
        game.finished,
        "Game should auto-finish when all players stand"
    );
}

#[test]
fn test_check_auto_finish_all_standing() {
    use blackjack_core::PlayerState;
    let mut game = test_game(vec!["player1@test.com", "player2@test.com"]).unwrap();

    game.players.get_mut("creator@test.com").unwrap().state = PlayerState::Standing;
    game.players.get_mut("player1@test.com").unwrap().state = PlayerState::Standing;
    game.players.get_mut("player2@test.com").unwrap().state = PlayerState::Standing;

    assert!(
        game.check_auto_finish(),
        "Should auto-finish when all players standing"
    );
}

#[test]
fn test_check_auto_finish_all_busted() {
    use blackjack_core::PlayerState;
    let mut game = test_game(vec!["player1@test.com", "player2@test.com"]).unwrap();

    game.players.get_mut("creator@test.com").unwrap().state = PlayerState::Busted;
    game.players.get_mut("player1@test.com").unwrap().state = PlayerState::Busted;
    game.players.get_mut("player2@test.com").unwrap().state = PlayerState::Busted;

    assert!(
        game.check_auto_finish(),
        "Should auto-finish when all players busted"
    );
}

#[test]
fn test_check_auto_finish_mixed() {
    use blackjack_core::PlayerState;
    let mut game = test_game(vec!["player1@test.com", "player2@test.com"]).unwrap();

    game.players.get_mut("creator@test.com").unwrap().state = PlayerState::Standing;
    game.players.get_mut("player1@test.com").unwrap().state = PlayerState::Standing;
    game.players.get_mut("player2@test.com").unwrap().state = PlayerState::Busted;

    assert!(
        game.check_auto_finish(),
        "Should auto-finish when all players done (standing or busted)"
    );
}

#[test]
fn test_check_auto_finish_has_active_player() {
    use blackjack_core::PlayerState;
    let mut game = test_game(vec!["player1@test.com", "player2@test.com"]).unwrap();

    game.players.get_mut("player1@test.com").unwrap().state = PlayerState::Standing;
    game.players.get_mut("player2@test.com").unwrap().state = PlayerState::Active;

    assert!(
        !game.check_auto_finish(),
        "Should not auto-finish with active players"
    );
}

#[test]
fn test_can_player_act_current_turn() {
    let game = test_game(vec!["player1@test.com", "player2@test.com"]).unwrap();

    assert!(
        game.can_player_act("creator@test.com"),
        "Creator (current player) should be able to act"
    );
    assert!(
        !game.can_player_act("player1@test.com"),
        "Non-current player should not be able to act"
    );
}

#[test]
fn test_can_player_act_enrollment_open() {
    let game = Game::new(test_creator_id(), test_creator_email(), 300).unwrap();

    // Enrollment still open
    assert!(
        !game.can_player_act(&test_creator_email()),
        "Cannot act during enrollment phase"
    );
}

#[test]
fn test_draw_card_advances_turn() {
    let mut game = test_game(vec!["player1@test.com", "player2@test.com"]).unwrap();

    assert_eq!(game.get_current_player(), Some("creator@test.com"));

    game.draw_card("creator@test.com").unwrap();

    assert_eq!(
        game.get_current_player(),
        Some("player1@test.com"),
        "Turn should advance after drawing"
    );
}

#[test]
fn test_draw_card_not_your_turn() {
    let mut game = test_game(vec!["player1@test.com", "player2@test.com"]).unwrap();

    assert_eq!(game.get_current_player(), Some("creator@test.com"));

    let result = game.draw_card("player1@test.com");
    assert_eq!(result, Err(GameError::NotPlayerTurn));
}

#[test]
fn test_draw_card_enrollment_open() {
    let mut game = Game::new(test_creator_id(), test_creator_email(), 300).unwrap();

    let result = game.draw_card(&test_creator_email());
    assert_eq!(
        result,
        Err(GameError::EnrollmentNotClosed),
        "Cannot draw during enrollment phase"
    );
}

#[test]
fn test_busted_player_state_updates() {
    use blackjack_core::PlayerState;
    let mut game = test_game(vec!["player1@test.com"]).unwrap();

    // Find high-value cards to make player bust
    let player = game.players.get_mut("player1@test.com").unwrap();

    // Add cards manually to make player bust (e.g., 10 + 10 + 5 = 25)
    let ten_card_1 = game
        .available_cards
        .iter()
        .find(|c| c.value == 10)
        .cloned()
        .expect("Should have a 10-value card");

    let ten_card_2 = game
        .available_cards
        .iter()
        .filter(|c| c.value == 10 && c.id != ten_card_1.id)
        .next()
        .cloned()
        .expect("Should have another 10-value card");

    let five_card = game
        .available_cards
        .iter()
        .find(|c| c.name == "5")
        .cloned()
        .expect("Should have a 5 card");

    player.add_card(ten_card_1);
    player.add_card(ten_card_2);
    player.add_card(five_card);

    assert_eq!(
        player.state,
        PlayerState::Busted,
        "Busted state should be set when points > 21"
    );
    assert!(player.busted, "Player should be marked as busted");
    assert!(player.points > 21, "Player points should be > 21");
}

// =====================================
// DEALER LOGIC TESTS
// =====================================

#[test]
fn test_dealer_plays_automatically_after_all_players_finish() {
    use blackjack_core::PlayerState;
    let mut game = test_game(vec!["player1@test.com"]).unwrap();

    // Both players stand
    game.stand("creator@test.com").unwrap();
    game.stand("player1@test.com").unwrap();

    // Game should be finished and dealer should have played
    assert!(game.finished, "Game should be finished");
    assert!(
        game.dealer.state == PlayerState::Standing || game.dealer.state == PlayerState::Busted,
        "Dealer should have played"
    );
}

#[test]
fn test_dealer_draws_until_17() {
    let mut game = test_game(vec!["player1@test.com"]).unwrap();

    // Manually trigger dealer play
    game.play_dealer().unwrap();

    // Dealer should have at least 17 points or be busted
    assert!(
        game.dealer.points >= 17 || game.dealer.busted,
        "Dealer should draw until 17 or bust"
    );
}

#[test]
fn test_dealer_stops_at_17_or_higher() {
    let mut game = test_game(vec!["player1@test.com"]).unwrap();

    game.play_dealer().unwrap();

    // If dealer didn't bust, they should have 17-21 points
    if !game.dealer.busted {
        assert!(
            game.dealer.points >= 17 && game.dealer.points <= 21,
            "Dealer should stop at 17-21"
        );
    }
}

#[test]
fn test_dealer_can_bust() {
    // Run multiple times to increase chance of dealer busting
    let mut busted_at_least_once = false;

    for _ in 0..50 {
        let mut game = test_game(vec!["player1@test.com"]).unwrap();
        game.play_dealer().unwrap();

        if game.dealer.busted {
            busted_at_least_once = true;
            assert!(
                game.dealer.points > 21,
                "Busted dealer should have > 21 points"
            );
            assert_eq!(
                game.dealer.state,
                PlayerState::Busted,
                "Dealer state should be Busted"
            );
            break;
        }
    }

    // This is probabilistic but should happen in 50 tries
    assert!(
        busted_at_least_once,
        "Dealer should bust at least once in 50 games"
    );
}

#[test]
fn test_dealer_marked_as_standing_when_not_busted() {
    use blackjack_core::PlayerState;

    // Run multiple times to get a non-busted dealer
    for _ in 0..20 {
        let mut game = test_game(vec!["player1@test.com"]).unwrap();
        game.play_dealer().unwrap();

        if !game.dealer.busted {
            assert_eq!(
                game.dealer.state,
                PlayerState::Standing,
                "Non-busted dealer should be Standing"
            );
            return; // Test passed
        }
    }
}

#[test]
fn test_dealer_included_in_game_results() {
    let mut game = test_game(vec!["player1@test.com"]).unwrap();

    // Set some points for testing
    game.players.get_mut("creator@test.com").unwrap().points = 18;
    game.players.get_mut("player1@test.com").unwrap().points = 20;

    game.play_dealer().unwrap();
    game.finished = true;

    let results = game.calculate_results();

    // Dealer should be in all_players
    assert!(
        results.all_players.contains_key("dealer"),
        "Results should include dealer"
    );

    let dealer_summary = results.all_players.get("dealer").unwrap();
    assert_eq!(
        dealer_summary.points, game.dealer.points,
        "Dealer points should match"
    );
}

#[test]
fn test_players_win_when_dealer_busts() {
    let mut game = test_game(vec!["player1@test.com"]).unwrap();

    // Set player points
    game.players.get_mut("creator@test.com").unwrap().points = 15;
    game.players.get_mut("player1@test.com").unwrap().points = 18;

    // Force dealer to bust by giving high-value cards
    for _ in 0..3 {
        if let Some(ten_card) = game
            .available_cards
            .iter()
            .position(|c| c.value == 10)
            .map(|idx| game.available_cards.remove(idx))
        {
            game.dealer.add_card(ten_card);
        }
    }

    assert!(game.dealer.busted, "Dealer should be busted");

    game.finished = true;
    let results = game.calculate_results();

    // Player with 18 points should win when dealer busts
    assert!(
        results.winner == Some("player1@test.com".to_string())
            || results.tied_players.contains(&"player1@test.com".to_string()),
        "Non-busted players should win when dealer busts"
    );
    assert_eq!(results.highest_score, 18, "Highest score should be 18");
}

#[test]
fn test_dealer_wins_when_players_have_lower_scores() {
    let mut game = test_game(vec!["player1@test.com"]).unwrap();

    // Set player points lower than dealer
    game.players.get_mut("creator@test.com").unwrap().points = 15;
    game.players.get_mut("player1@test.com").unwrap().points = 16;

    // Manually set dealer to 20
    while game.dealer.points < 20 {
        if let Some(card_idx) = game.available_cards.iter().position(|c| c.value > 0) {
            let card = game.available_cards.remove(card_idx);
            game.dealer.add_card(card);
            if game.dealer.points >= 20 {
                break;
            }
        }
    }

    // Ensure dealer didn't bust and has a good score
    if !game.dealer.busted && game.dealer.points >= 17 && game.dealer.points <= 21 {
        game.finished = true;
        let results = game.calculate_results();

        // No player should win if dealer has higher score
        if game.dealer.points > 16 {
            assert!(
                results.winner.is_none() || results.highest_score <= game.dealer.points,
                "Dealer should win with higher score"
            );
        }
    }
}

#[test]
fn test_push_when_player_ties_dealer() {
    let mut game = test_game(vec!["player1@test.com"]).unwrap();

    // Set both to 19
    game.players.get_mut("creator@test.com").unwrap().points = 19;

    // Manually set dealer to 19
    while game.dealer.points < 19 {
        if let Some(card_idx) = game.available_cards.iter().position(|c| c.value > 0) {
            let card = game.available_cards.remove(card_idx);
            game.dealer.add_card(card);
            if game.dealer.points >= 19 {
                break;
            }
        }
    }

    // Set other player to bust
    game.players.get_mut("player1@test.com").unwrap().points = 25;
    game.players.get_mut("player1@test.com").unwrap().busted = true;

    if game.dealer.points == 19 && !game.dealer.busted {
        game.finished = true;
        let results = game.calculate_results();

        // Creator tied with dealer (push), should not be counted as winner
        assert!(
            results.winner.is_none(),
            "Push should not result in a winner"
        );
        assert_eq!(results.highest_score, 0, "No wins when only pushes exist");
    }
}

#[test]
fn test_dealer_cannot_play_after_game_finished() {
    let mut game = test_game(vec!["player1@test.com"]).unwrap();

    game.play_dealer().unwrap();
    game.finished = true;

    let result = game.play_dealer();
    assert_eq!(
        result,
        Err(GameError::GameAlreadyFinished),
        "Cannot play dealer after game finished"
    );
}

#[test]
fn test_dealer_handles_empty_deck() {
    let mut game = test_game(vec!["player1@test.com"]).unwrap();

    // Empty the deck
    game.available_cards.clear();

    let result = game.play_dealer();
    assert_eq!(
        result,
        Err(GameError::DeckEmpty),
        "Should return DeckEmpty error"
    );
}

// ============================================================================
// Game Results & Scoring Tests
// ============================================================================

#[test]
fn test_result_player_beats_dealer() {
    let mut game = test_game(vec!["player1@test.com"]).unwrap();

    // Set up scenario: player1 has 20, dealer has 18
    game.players.get_mut("player1@test.com").unwrap().points = 20;
    game.dealer.points = 18;
    game.dealer.busted = false;
    game.finished = true;

    let results = game.calculate_results();

    // Check winner fields (backward compatibility)
    assert_eq!(results.winner, Some("player1@test.com".to_string()));
    assert_eq!(results.highest_score, 20);
    assert!(results.tied_players.is_empty());

    // Check new detailed fields
    assert_eq!(results.dealer_points, 18);
    assert_eq!(results.dealer_busted, false);

    let player1_result = results.player_results.get("player1@test.com").unwrap();
    assert_eq!(player1_result.points, 20);
    assert!(!player1_result.busted);
    assert!(matches!(
        player1_result.outcome,
        blackjack_core::PlayerOutcome::Won
    ));
}

#[test]
fn test_result_dealer_beats_player() {
    let mut game = test_game(vec!["player1@test.com"]).unwrap();

    // Set up scenario: dealer has 20, player1 has 18
    game.players.get_mut("player1@test.com").unwrap().points = 18;
    game.dealer.points = 20;
    game.dealer.busted = false;
    game.finished = true;

    let results = game.calculate_results();

    // Check winner fields
    assert!(results.winner.is_none());
    assert_eq!(results.highest_score, 0);
    assert!(results.tied_players.is_empty());

    // Check detailed fields
    assert_eq!(results.dealer_points, 20);
    assert_eq!(results.dealer_busted, false);

    let player1_result = results.player_results.get("player1@test.com").unwrap();
    assert_eq!(player1_result.points, 18);
    assert!(!player1_result.busted);
    assert!(matches!(
        player1_result.outcome,
        blackjack_core::PlayerOutcome::Lost
    ));
}

#[test]
fn test_result_push_tie_with_dealer() {
    let mut game = test_game(vec!["player1@test.com"]).unwrap();

    // Set up scenario: both have 19 (push)
    game.players.get_mut("player1@test.com").unwrap().points = 19;
    game.dealer.points = 19;
    game.dealer.busted = false;
    game.finished = true;

    let results = game.calculate_results();

    // Check winner fields
    assert!(results.winner.is_none());
    assert_eq!(results.highest_score, 0);
    assert!(results.tied_players.is_empty());

    // Check detailed fields
    let player1_result = results.player_results.get("player1@test.com").unwrap();
    assert_eq!(player1_result.points, 19);
    assert!(!player1_result.busted);
    assert!(matches!(
        player1_result.outcome,
        blackjack_core::PlayerOutcome::Push
    ));
}

#[test]
fn test_result_player_busted() {
    let mut game = test_game(vec!["player1@test.com"]).unwrap();

    // Set up scenario: player1 busted
    game.players.get_mut("player1@test.com").unwrap().points = 25;
    game.players.get_mut("player1@test.com").unwrap().busted = true;
    game.dealer.points = 18;
    game.dealer.busted = false;
    game.finished = true;

    let results = game.calculate_results();

    // Check winner fields
    assert!(results.winner.is_none());

    // Check detailed fields
    let player1_result = results.player_results.get("player1@test.com").unwrap();
    assert_eq!(player1_result.points, 25);
    assert!(player1_result.busted);
    assert!(matches!(
        player1_result.outcome,
        blackjack_core::PlayerOutcome::Busted
    ));
}

#[test]
fn test_result_dealer_busted_players_win() {
    let mut game = test_game(vec!["player1@test.com", "player2@test.com"]).unwrap();

    // Set up scenario: dealer busted, both players have different scores
    game.players.get_mut("player1@test.com").unwrap().points = 18;
    game.players.get_mut("player2@test.com").unwrap().points = 16;
    game.dealer.points = 25;
    game.dealer.busted = true;
    game.finished = true;

    let results = game.calculate_results();

    // Check winner fields - player1 has highest score
    assert_eq!(results.winner, Some("player1@test.com".to_string()));
    assert_eq!(results.highest_score, 18);
    assert_eq!(results.dealer_busted, true);

    // Check that all non-busted players won
    let player1_result = results.player_results.get("player1@test.com").unwrap();
    assert!(matches!(
        player1_result.outcome,
        blackjack_core::PlayerOutcome::Won
    ));

    let player2_result = results.player_results.get("player2@test.com").unwrap();
    assert!(matches!(
        player2_result.outcome,
        blackjack_core::PlayerOutcome::Won
    ));
}

#[test]
fn test_result_mixed_outcomes() {
    let mut game = test_game(vec!["player1@test.com", "player2@test.com", "player3@test.com"])
        .unwrap();

    // Set up scenario:
    // - player1: 20 (wins)
    // - player2: 18 (loses)
    // - player3: 19 (push)
    // - dealer: 19
    game.players.get_mut("player1@test.com").unwrap().points = 20;
    game.players.get_mut("player2@test.com").unwrap().points = 18;
    game.players.get_mut("player3@test.com").unwrap().points = 19;
    game.dealer.points = 19;
    game.dealer.busted = false;
    game.finished = true;

    let results = game.calculate_results();

    // Check winner
    assert_eq!(results.winner, Some("player1@test.com".to_string()));
    assert_eq!(results.highest_score, 20);

    // Check individual outcomes
    let player1_result = results.player_results.get("player1@test.com").unwrap();
    assert!(matches!(
        player1_result.outcome,
        blackjack_core::PlayerOutcome::Won
    ));

    let player2_result = results.player_results.get("player2@test.com").unwrap();
    assert!(matches!(
        player2_result.outcome,
        blackjack_core::PlayerOutcome::Lost
    ));

    let player3_result = results.player_results.get("player3@test.com").unwrap();
    assert!(matches!(
        player3_result.outcome,
        blackjack_core::PlayerOutcome::Push
    ));
}

#[test]
fn test_result_all_players_bust() {
    let mut game = test_game(vec!["player1@test.com", "player2@test.com"]).unwrap();

    // All players busted
    game.players.get_mut("player1@test.com").unwrap().points = 25;
    game.players.get_mut("player1@test.com").unwrap().busted = true;
    game.players.get_mut("player2@test.com").unwrap().points = 23;
    game.players.get_mut("player2@test.com").unwrap().busted = true;
    game.dealer.points = 18;
    game.dealer.busted = false;
    game.finished = true;

    let results = game.calculate_results();

    // No winner
    assert!(results.winner.is_none());
    assert_eq!(results.highest_score, 0);

    // Both players busted
    let player1_result = results.player_results.get("player1@test.com").unwrap();
    assert!(matches!(
        player1_result.outcome,
        blackjack_core::PlayerOutcome::Busted
    ));

    let player2_result = results.player_results.get("player2@test.com").unwrap();
    assert!(matches!(
        player2_result.outcome,
        blackjack_core::PlayerOutcome::Busted
    ));
}

#[test]
fn test_result_tied_winners() {
    let mut game = test_game(vec!["player1@test.com", "player2@test.com"]).unwrap();

    // Both players tie with same winning score
    game.players.get_mut("player1@test.com").unwrap().points = 20;
    game.players.get_mut("player2@test.com").unwrap().points = 20;
    game.dealer.points = 18;
    game.dealer.busted = false;
    game.finished = true;

    let results = game.calculate_results();

    // No single winner, but tied_players should have both
    assert!(results.winner.is_none());
    assert_eq!(results.tied_players.len(), 2);
    assert!(results.tied_players.contains(&"player1@test.com".to_string()));
    assert!(results.tied_players.contains(&"player2@test.com".to_string()));
    assert_eq!(results.highest_score, 20);

    // Both should show as Won
    let player1_result = results.player_results.get("player1@test.com").unwrap();
    assert!(matches!(
        player1_result.outcome,
        blackjack_core::PlayerOutcome::Won
    ));

    let player2_result = results.player_results.get("player2@test.com").unwrap();
    assert!(matches!(
        player2_result.outcome,
        blackjack_core::PlayerOutcome::Won
    ));
}

#[test]
fn test_result_multiple_players_tie_and_lose() {
    let mut game = test_game(vec!["player1@test.com", "player2@test.com"]).unwrap();

    // Both players tie at 18, dealer has 20 (both lose)
    game.players.get_mut("player1@test.com").unwrap().points = 18;
    game.players.get_mut("player2@test.com").unwrap().points = 18;
    game.dealer.points = 20;
    game.dealer.busted = false;
    game.finished = true;

    let results = game.calculate_results();

    // No winner since both lost
    assert!(results.winner.is_none());
    assert_eq!(results.highest_score, 0);
    assert!(results.tied_players.is_empty(), "tied_players should be empty when players tie but lose");

    // Both should show as Lost
    let player1_result = results.player_results.get("player1@test.com").unwrap();
    assert_eq!(player1_result.points, 18);
    assert!(matches!(
        player1_result.outcome,
        blackjack_core::PlayerOutcome::Lost
    ));

    let player2_result = results.player_results.get("player2@test.com").unwrap();
    assert_eq!(player2_result.points, 18);
    assert!(matches!(
        player2_result.outcome,
        blackjack_core::PlayerOutcome::Lost
    ));
}

#[test]
fn test_result_multiple_players_tie_and_push() {
    let mut game = test_game(vec!["player1@test.com", "player2@test.com", "player3@test.com"])
        .unwrap();

    // All three players tie at 19, dealer also has 19 (all push)
    game.players.get_mut("player1@test.com").unwrap().points = 19;
    game.players.get_mut("player2@test.com").unwrap().points = 19;
    game.players.get_mut("player3@test.com").unwrap().points = 19;
    game.dealer.points = 19;
    game.dealer.busted = false;
    game.finished = true;

    let results = game.calculate_results();

    // No winner since all pushed
    assert!(results.winner.is_none());
    assert_eq!(results.highest_score, 0);
    assert!(results.tied_players.is_empty(), "tied_players should be empty when all push");

    // All should show as Push
    let player1_result = results.player_results.get("player1@test.com").unwrap();
    assert_eq!(player1_result.points, 19);
    assert!(matches!(
        player1_result.outcome,
        blackjack_core::PlayerOutcome::Push
    ));

    let player2_result = results.player_results.get("player2@test.com").unwrap();
    assert_eq!(player2_result.points, 19);
    assert!(matches!(
        player2_result.outcome,
        blackjack_core::PlayerOutcome::Push
    ));

    let player3_result = results.player_results.get("player3@test.com").unwrap();
    assert_eq!(player3_result.points, 19);
    assert!(matches!(
        player3_result.outcome,
        blackjack_core::PlayerOutcome::Push
    ));
}

#[test]
fn test_result_three_players_tie_and_win() {
    let mut game = test_game(vec!["player1@test.com", "player2@test.com", "player3@test.com"])
        .unwrap();

    // All three players tie at 20, dealer has 18 (all win)
    game.players.get_mut("player1@test.com").unwrap().points = 20;
    game.players.get_mut("player2@test.com").unwrap().points = 20;
    game.players.get_mut("player3@test.com").unwrap().points = 20;
    game.dealer.points = 18;
    game.dealer.busted = false;
    game.finished = true;

    let results = game.calculate_results();

    // No single winner, all three should be in tied_players
    assert!(results.winner.is_none());
    assert_eq!(results.tied_players.len(), 3);
    assert!(results.tied_players.contains(&"player1@test.com".to_string()));
    assert!(results.tied_players.contains(&"player2@test.com".to_string()));
    assert!(results.tied_players.contains(&"player3@test.com".to_string()));
    assert_eq!(results.highest_score, 20);

    // All should show as Won
    for email in ["player1@test.com", "player2@test.com", "player3@test.com"] {
        let player_result = results.player_results.get(email).unwrap();
        assert_eq!(player_result.points, 20);
        assert!(matches!(
            player_result.outcome,
            blackjack_core::PlayerOutcome::Won
        ));
    }
}
