use blackjack_core::{Game, GameError};
use uuid::Uuid;

// Helper function to create a test creator_id
fn test_creator_id() -> Uuid {
    Uuid::new_v4()
}

#[test]
fn test_deck_has_52_cards() {
    let game = Game::new(test_creator_id(), vec!["player1@test.com".to_string()]).unwrap();
    assert_eq!(game.available_cards.len(), 52, "Deck should have exactly 52 cards");
}

#[test]
fn test_four_cards_of_each_type() {
    let game = Game::new(test_creator_id(), vec!["player1@test.com".to_string()]).unwrap();
    
    // Count cards by name
    let card_types = ["A", "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K"];
    for card_type in card_types.iter() {
        let count = game.available_cards.iter().filter(|c| c.name == *card_type).count();
        assert_eq!(count, 4, "Should have exactly 4 cards of type {}", card_type);
    }
}

#[test]
fn test_cards_have_correct_suits() {
    let game = Game::new(test_creator_id(), vec!["player1@test.com".to_string()]).unwrap();
    
    let suits = ["Hearts", "Diamonds", "Clubs", "Spades"];
    for suit in suits.iter() {
        let count = game.available_cards.iter().filter(|c| c.suit == *suit).count();
        assert_eq!(count, 13, "Should have exactly 13 cards of suit {}", suit);
    }
}

#[test]
fn test_deck_exhaustion() {
    let mut game = Game::new(test_creator_id(), vec!["player1@test.com".to_string()]).unwrap();
    
    // Draw cards until player busts or game finishes
    let mut cards_drawn = 0;
    loop {
        let result = game.draw_card("player1@test.com");
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
                // Game finished (auto-finish after player busts)
                break;
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
        
        if cards_drawn >= 52 {
            break;
        }
    }
    
    // Verify we can create a game and test DeckEmpty by manually emptying deck
    let mut game2 = Game::new(test_creator_id(), vec!["player1@test.com".to_string()]).unwrap();
    
    // Manually empty the deck
    game2.available_cards.clear();
    
    // Now try to draw from empty deck
    let result = game2.draw_card("player1@test.com");
    assert_eq!(result, Err(GameError::DeckEmpty), "Should return DeckEmpty error");
}

#[test]
fn test_ace_value_can_be_changed_multiple_times() {
    let mut game = Game::new(test_creator_id(), vec!["player1@test.com".to_string()]).unwrap();
    
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
    let mut game = Game::new(test_creator_id(), vec!["player1@test.com".to_string()]).unwrap();
    
    game.finish_game();
    
    let result = game.draw_card("player1@test.com");
    assert_eq!(result, Err(GameError::GameAlreadyFinished), "Cannot draw after game finished");
}

#[test]
fn test_game_finished_prevents_ace_change() {
    let mut game = Game::new(test_creator_id(), vec!["player1@test.com".to_string()]).unwrap();
    
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
    let game = Game::new(test_creator_id(), vec!["player1@test.com".to_string()]).unwrap();
    
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
    let mut game = Game::new(test_creator_id(), vec![
        "player1@test.com".to_string(),
        "player2@test.com".to_string(),
        "player3@test.com".to_string(),
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
    let mut game = Game::new(test_creator_id(), vec![
        "player1@test.com".to_string(),
        "player2@test.com".to_string(),
        "player3@test.com".to_string(),
    ])
    .unwrap();
    
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
    let mut game = Game::new(test_creator_id(), vec![
        "player1@test.com".to_string(),
        "player2@test.com".to_string(),
    ])
    .unwrap();
    
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
    let mut game = Game::new(test_creator_id(), vec![
        "player1@test.com".to_string(),
        "player2@test.com".to_string(),
        "player3@test.com".to_string(),
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
    let result = Game::new(test_creator_id(), vec![]);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), GameError::InvalidPlayerCount));
}

#[test]
fn test_invalid_player_count_too_many() {
    let players: Vec<String> = (1..=11).map(|i| format!("player{}@test.com", i)).collect();
    let result = Game::new(test_creator_id(), players);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), GameError::InvalidPlayerCount));
}

#[test]
fn test_invalid_email_empty() {
    let result = Game::new(test_creator_id(), vec!["".to_string()]);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), GameError::InvalidEmail));
}

#[test]
fn test_invalid_email_duplicate() {
    let result = Game::new(test_creator_id(), vec![
        "player1@test.com".to_string(),
        "player1@test.com".to_string(),
    ]);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), GameError::InvalidEmail));
}

#[test]
fn test_player_not_in_game() {
    let mut game = Game::new(test_creator_id(), vec!["player1@test.com".to_string()]).unwrap();
    
    let result = game.draw_card("nonexistent@test.com");
    // With turn management, it will return NotPlayerTurn before checking if player exists
    assert_eq!(result, Err(GameError::NotPlayerTurn));
}

#[test]
fn test_busted_player_cannot_draw() {
    let mut game = Game::new(test_creator_id(), vec!["player1@test.com".to_string()]).unwrap();
    
    // Manually set player as busted
    game.players.get_mut("player1@test.com").unwrap().busted = true;
    
    let result = game.draw_card("player1@test.com");
    assert_eq!(result, Err(GameError::PlayerAlreadyBusted));
}

#[test]
fn test_valid_player_range() {
    for count in 1..=10 {
        let players: Vec<String> = (1..=count).map(|i| format!("player{}@test.com", i)).collect();
        let result = Game::new(test_creator_id(), players);
        assert!(result.is_ok(), "Should accept {} players", count);
    }
}
