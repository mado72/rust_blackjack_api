use blackjack_core::{Game, GameError};
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
    assert_eq!(game.available_cards.len(), 52, "Deck should have exactly 52 cards");
}

#[test]
fn test_four_cards_of_each_type() {
    let game = test_game(vec!["player1@test.com"]).unwrap();
    
    // Count cards by name
    let card_types = ["A", "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K"];
    for card_type in card_types.iter() {
        let count = game.available_cards.iter().filter(|c| c.name == *card_type).count();
        assert_eq!(count, 4, "Should have exactly 4 cards of type {}", card_type);
    }
}

#[test]
fn test_cards_have_correct_suits() {
    let game = test_game(vec!["player1@test.com"]).unwrap();
    
    let suits = ["Hearts", "Diamonds", "Clubs", "Spades"];
    for suit in suits.iter() {
        let count = game.available_cards.iter().filter(|c| c.suit == *suit).count();
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
    assert_eq!(result, Err(GameError::DeckEmpty), "Should return DeckEmpty error");
}

#[test]
fn test_ace_value_can_be_changed_multiple_times() {
    let mut game = test_game(vec!["player1@test.com"]).unwrap();
    
    // Find an Ace in the deck and give it to the player
    let ace_index = game.available_cards.iter().position(|c| c.name == "A").expect("Should have an Ace in deck");
    let ace = game.available_cards.remove(ace_index);
    let ace_id = ace.id;
    
    game.players.get_mut("player1@test.com").unwrap().add_card(ace);
    
    let player = game.players.get("player1@test.com").unwrap();
    let initial_points = player.points;
    
    // Change Ace to 11
    game.set_ace_value("player1@test.com", ace_id, true).unwrap();
    let player = game.players.get("player1@test.com").unwrap();
    assert_eq!(player.points, initial_points + 10, "Points should increase by 10 when Ace is 11");
    
    // Change back to 1
    game.set_ace_value("player1@test.com", ace_id, false).unwrap();
    let player = game.players.get("player1@test.com").unwrap();
    assert_eq!(player.points, initial_points, "Points should return to initial when Ace is 1");
    
    // Change to 11 again
    game.set_ace_value("player1@test.com", ace_id, true).unwrap();
    let player = game.players.get("player1@test.com").unwrap();
    assert_eq!(player.points, initial_points + 10, "Can change Ace value multiple times");
}

#[test]
fn test_game_finished_prevents_draw() {
    let mut game = test_game(vec!["player1@test.com"]).unwrap();
    
    game.finish_game();
    
    let result = game.draw_card("player1@test.com");
    assert_eq!(result, Err(GameError::GameAlreadyFinished), "Cannot draw after game finished");
}

#[test]
fn test_game_finished_prevents_ace_change() {
    let mut game = test_game(vec!["player1@test.com"]).unwrap();
    
    // Find an Ace and give it to the player
    let ace_index = game.available_cards.iter().position(|c| c.name == "A").expect("Should have an Ace in deck");
    let ace = game.available_cards.remove(ace_index);
    let ace_id = ace.id;
    
    game.players.get_mut("player1@test.com").unwrap().add_card(ace);
    
    game.finish_game();
    
    let result = game.set_ace_value("player1@test.com", ace_id, true);
    assert_eq!(result, Err(GameError::GameAlreadyFinished), "Cannot change Ace after game finished");
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
    assert_eq!(game.available_cards.len(), deserialized.available_cards.len());
    assert_eq!(game.finished, deserialized.finished);
}

#[test]
fn test_calculate_results_single_winner() {
    let mut game = test_game(vec![
        "player1@test.com",
        "player2@test.com",
        "player3@test.com",
    ]).unwrap();
    
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
    ]).unwrap();
    
    game.players.get_mut("player1@test.com").unwrap().points = 19;
    game.players.get_mut("player2@test.com").unwrap().points = 15;
    game.players.get_mut("player3@test.com").unwrap().points = 19;
    
    let result = game.calculate_results();
    
    assert_eq!(result.winner, None);
    assert_eq!(result.highest_score, 19);
    assert_eq!(result.tied_players.len(), 2);
    assert!(result.tied_players.contains(&"player1@test.com".to_string()));
    assert!(result.tied_players.contains(&"player3@test.com".to_string()));
}

#[test]
fn test_calculate_results_all_bust() {
    let mut game = test_game(vec![
        "player1@test.com",
        "player2@test.com",
    ]).unwrap();
    
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
    ]).unwrap();
    
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
    assert!(result.is_ok(), "Game should be created with creator enrolled");
    
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
            assert!(game.add_player(email).is_ok(), "Should allow up to 10 total players (9 + creator)");
        } else {
            assert!(game.add_player(email).is_err(), "Should reject more than 10 players");
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
        assert!(result.is_ok(), "Should accept game creation with valid email");
        
        if let Ok(game) = result {
            assert_eq!(game.players.len(), 1, "Game should start with 1 player (creator)");
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
    assert_eq!(creator.state, PlayerState::Active, "Creator should be Active");
    
    let player = game.players.get("player1@test.com").unwrap();
    assert_eq!(player.state, PlayerState::Active, "New players should be Active");
}

#[test]
fn test_get_current_player() {
    let game = test_game(vec![
        "player1@test.com",
        "player2@test.com",
        "player3@test.com",
    ]).unwrap();
    
    // Creator is enrolled first, so should have first turn
    let current = game.get_current_player();
    assert_eq!(current, Some("creator@test.com"), "Creator should have first turn");
}

#[test]
fn test_advance_turn() {
    let mut game = test_game(vec![
        "player1@test.com",
        "player2@test.com",
        "player3@test.com",
    ]).unwrap();
    
    assert_eq!(game.get_current_player(), Some("creator@test.com"));
    
    game.advance_turn();
    assert_eq!(game.get_current_player(), Some("player1@test.com"));
    
    game.advance_turn();
    assert_eq!(game.get_current_player(), Some("player2@test.com"));
    
    game.advance_turn();
    assert_eq!(game.get_current_player(), Some("player3@test.com"));
    
    game.advance_turn();
    assert_eq!(game.get_current_player(), Some("creator@test.com"), "Should wrap around to creator");
}

#[test]
fn test_advance_turn_skips_standing_players() {
    use blackjack_core::PlayerState;
    let mut game = test_game(vec![
        "player1@test.com",
        "player2@test.com",
        "player3@test.com",
    ]).unwrap();
    
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
    ]).unwrap();
    
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
    assert!(!game.finished, "Game should not finish with players remaining");
    
    game.stand("player1@test.com").unwrap();
    assert!(!game.finished, "Game should not finish with one player remaining");
    
    game.stand("player2@test.com").unwrap();
    assert!(game.finished, "Game should auto-finish when all players stand");
}

#[test]
fn test_check_auto_finish_all_standing() {
    use blackjack_core::PlayerState;
    let mut game = test_game(vec!["player1@test.com", "player2@test.com"]).unwrap();
    
    game.players.get_mut("creator@test.com").unwrap().state = PlayerState::Standing;
    game.players.get_mut("player1@test.com").unwrap().state = PlayerState::Standing;
    game.players.get_mut("player2@test.com").unwrap().state = PlayerState::Standing;
    
    assert!(game.check_auto_finish(), "Should auto-finish when all players standing");
}

#[test]
fn test_check_auto_finish_all_busted() {
    use blackjack_core::PlayerState;
    let mut game = test_game(vec!["player1@test.com", "player2@test.com"]).unwrap();
    
    game.players.get_mut("creator@test.com").unwrap().state = PlayerState::Busted;
    game.players.get_mut("player1@test.com").unwrap().state = PlayerState::Busted;
    game.players.get_mut("player2@test.com").unwrap().state = PlayerState::Busted;
    
    assert!(game.check_auto_finish(), "Should auto-finish when all players busted");
}

#[test]
fn test_check_auto_finish_mixed() {
    use blackjack_core::PlayerState;
    let mut game = test_game(vec!["player1@test.com", "player2@test.com"]).unwrap();
    
    game.players.get_mut("creator@test.com").unwrap().state = PlayerState::Standing;
    game.players.get_mut("player1@test.com").unwrap().state = PlayerState::Standing;
    game.players.get_mut("player2@test.com").unwrap().state = PlayerState::Busted;
    
    assert!(game.check_auto_finish(), "Should auto-finish when all players done (standing or busted)");
}

#[test]
fn test_check_auto_finish_has_active_player() {
    use blackjack_core::PlayerState;
    let mut game = test_game(vec!["player1@test.com", "player2@test.com"]).unwrap();
    
    game.players.get_mut("player1@test.com").unwrap().state = PlayerState::Standing;
    game.players.get_mut("player2@test.com").unwrap().state = PlayerState::Active;
    
    assert!(!game.check_auto_finish(), "Should not auto-finish with active players");
}

#[test]
fn test_can_player_act_current_turn() {
    let game = test_game(vec!["player1@test.com", "player2@test.com"]).unwrap();
    
    assert!(game.can_player_act("creator@test.com"), "Creator (current player) should be able to act");
    assert!(!game.can_player_act("player1@test.com"), "Non-current player should not be able to act");
}

#[test]
fn test_can_player_act_enrollment_open() {
    let game = Game::new(test_creator_id(), test_creator_email(), 300).unwrap();
    
    // Enrollment still open
    assert!(!game.can_player_act(&test_creator_email()), "Cannot act during enrollment phase");
}

#[test]
fn test_draw_card_advances_turn() {
    let mut game = test_game(vec!["player1@test.com", "player2@test.com"]).unwrap();
    
    assert_eq!(game.get_current_player(), Some("creator@test.com"));
    
    game.draw_card("creator@test.com").unwrap();
    
    assert_eq!(game.get_current_player(), Some("player1@test.com"), "Turn should advance after drawing");
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
    assert_eq!(result, Err(GameError::NotPlayerTurn), "Cannot draw during enrollment phase");
}

#[test]
fn test_busted_player_state_updates() {
    use blackjack_core::PlayerState;
    let mut game = test_game(vec!["player1@test.com"]).unwrap();
    
    // Find high-value cards to make player bust
    let player = game.players.get_mut("player1@test.com").unwrap();
    
    // Add cards manually to make player bust (e.g., 10 + 10 + 5 = 25)
    let ten_card_1 = game.available_cards.iter()
        .find(|c| c.value == 10)
        .cloned()
        .expect("Should have a 10-value card");
    
    let ten_card_2 = game.available_cards.iter()
        .filter(|c| c.value == 10 && c.id != ten_card_1.id)
        .next()
        .cloned()
        .expect("Should have another 10-value card");
    
    let five_card = game.available_cards.iter()
        .find(|c| c.name == "5")
        .cloned()
        .expect("Should have a 5 card");
    
    player.add_card(ten_card_1);
    player.add_card(ten_card_2);
    player.add_card(five_card);
    
    assert_eq!(player.state, PlayerState::Busted, "Busted state should be set when points > 21");
    assert!(player.busted, "Player should be marked as busted");
    assert!(player.points > 21, "Player points should be > 21");
}
