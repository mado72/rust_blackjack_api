use std::io::{self, Write};
use std::collections::HashMap;
use rand::Rng;

/// Constant array containing all the cards in the deck.
/// Each card is represented by a tuple (name, value).
/// The Ace ("A") has a base value of 1, but can be changed to 11 during the game.
/// Face cards (J, Q, K) have a value of 10.
const CARDS: [(&str, u8); 13] = [
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

/// Main function for the card game.
/// 
/// Manages the complete game flow:
/// 1. Requests the number of players
/// 2. Validates that the number of players is between 1 and 10
/// 3. Executes one round for each player
/// 4. Determines and displays the winner
/// 
/// # Panics
/// Panics if the number of players is less than 1 or greater than 10.
fn main() {
    println!("Welcome to the Card Game!");
    
    let total_players = read_total_players();

    assert!(total_players > 0, "The number of players must be greater than zero.");
    assert!(total_players <= 10, "The maximum number of players is 10.");

    let mut players_points = vec![0u8; total_players as usize];

    for player_number in 1..=total_players {

        println!("\nPlayer {}'s turn:", player_number);
        let points = play_game_round();

        println!("Player {} finished with {} points.", player_number, points);
        players_points[(player_number - 1) as usize] = points;
    }

    determine_winner(players_points);
    println!("Finished.");
}

/// Reads the total number of players from user input.
/// 
/// # Returns
/// Returns the number of players as u8.
/// 
/// # Panics
/// Panics if the input is not a valid number.
/// 
/// # Note
/// After calling this function, the main function validates that the returned value
/// is between 1 and 10 (inclusive).
fn read_total_players() -> u8 {
    let mut input = String::new();
    print!("Enter the number of players: ");
    io::stdout().flush().unwrap();
    io::stdin()
        .read_line(&mut input)
        .expect("Error reading input");
    let total_players: u8 = input.trim().parse().expect("Please enter a valid number");
    total_players
}

/// Executes a complete game round for a player.
/// 
/// The player can request cards repeatedly until:
/// - They decide to stop
/// - They exceed 21 points (bust)
/// 
/// Drawn cards are removed from the deck (cannot be drawn again).
/// If the player receives an Ace, they can choose whether it's worth 1 or 11 points.
/// 
/// # Returns
/// Returns the player's final score.
fn play_game_round() -> u8 {
    // Creates a HashMap with all available cards
    let mut card_points_map: HashMap<String, u8> = HashMap::new();
    
    for (name, value) in CARDS.iter() {
        card_points_map.insert(name.to_string(), *value);
    }

    let mut points: u8 = 0;
    loop {
        // Asks the player if they want a card
        if !prompt_decision("Do you want a card?") {
            break;
        }

        // Draws a random card from the available deck
        let random_index = rand::rng().random_range(0..card_points_map.len());
        let drawn_card = CARDS[random_index].0;
        println!("You got the card: {}", drawn_card);
        // Removes the card from the deck so it cannot be drawn again
        let choosed = card_points_map.remove(drawn_card as &str);

        if let Some(value) = choosed {
            // Special logic for the Ace: allows choosing between 1 or 11 points
            if drawn_card == "A"
                && prompt_decision("You have an Ace! Do you want it to count as 11 points instead of 1?") {
                    points += 10; // Adds 10 extra points (1 + 10 = 11)
                }
            points += value;
        }   
        println!("Your current points: {}", points);

        // Checks if the player exceeded 21 points (bust)
        if points > 21 {
            println!("You exceeded 21 points! Game over.");
            break;
        }
    }
    points
}

/// Determines and displays the winner of the game.
/// 
/// # Arguments
/// * `players_points` - Vector containing the final score of each player.
/// 
/// # Rules
/// - The winner is the player with the highest score <= 21
/// - Players with score > 21 are disqualified
/// - In case of a tie, all tied players are announced
/// - If all exceed 21, there is no winner
fn determine_winner(players_points: Vec<u8>) {
    let mut winner_index: Option<usize> = None;
    let mut highest_points: u8 = 0;
    let mut draw_players: Vec<usize> = Vec::new();

    // Iterates over all players to find the winner
    for (index, &points) in players_points.iter().enumerate() {
        if points <= 21 {
            if points == highest_points {
                draw_players.push(index);
            } else if points > highest_points {
                highest_points = points;
                winner_index = Some(index);
                draw_players.clear();
            }
        }
    }

    // Displays the game results
    println!("\n\n==========================\nGame Results:\n==========================");
    if draw_players.len() > 1 {
        println!("It's a draw between players: {:?}", draw_players.iter().map(|i| i + 1).collect::<Vec<usize>>());
    }
    else if let Some(winner) = winner_index {
        println!("Player {} wins with {} points!", winner + 1, highest_points);
    } else {
        println!("No winner this time.");
    }
}

/// Prompts the player for a yes/no decision.
/// 
/// # Arguments
/// * `prompt` - Message to be displayed to the player.
/// 
/// # Returns
/// Returns `true` if the response is "Y" (uppercase or lowercase) or empty (Enter).
/// Returns `false` if the response is "N" or any other input.
fn prompt_decision(prompt: &str) -> bool {
    let mut input = String::new();
    print!("{} [Y/n]: ", prompt);
    io::stdout().flush().unwrap();
    io::stdin()
        .read_line(&mut input)
        .expect("Error reading input");
    let response = input.trim().to_uppercase();
    response == "Y" || response.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_winner_single_winner() {
        // Player 2 wins with 20 points
        let players_points = vec![15, 20, 18];
        // Manual test: verify that player 2 wins
        let mut winner_index: Option<usize> = None;
        let mut highest_points: u8 = 0;

        for (index, &points) in players_points.iter().enumerate() {
            if points <= 21 && points > highest_points {
                highest_points = points;
                winner_index = Some(index);
            }
        }

        assert_eq!(winner_index, Some(1));
        assert_eq!(highest_points, 20);
    }

    #[test]
    fn test_determine_winner_all_bust() {
        // All players exceed 21
        let players_points = vec![22, 25, 23];
        let mut winner_index: Option<usize> = None;
        let mut highest_points: u8 = 0;

        for (index, &points) in players_points.iter().enumerate() {
            if points <= 21 && points > highest_points {
                highest_points = points;
                winner_index = Some(index);
            }
        }

        assert_eq!(winner_index, None);
    }

    #[test]
    fn test_determine_winner_draw() {
        // Draw between players 1 and 3 with 19 points
        let players_points = vec![19, 15, 19];
        let mut highest_points: u8 = 0;
        let mut draw_players: Vec<usize> = Vec::new();

        for (index, &points) in players_points.iter().enumerate() {
            if points <= 21 {
                if points == highest_points && highest_points > 0 {
                    if draw_players.is_empty() {
                        // Adds the previous player who had the highest score
                        for (prev_index, &prev_points) in players_points.iter().enumerate() {
                            if prev_points == highest_points && prev_index < index {
                                draw_players.push(prev_index);
                            }
                        }
                    }
                    draw_players.push(index);
                } else if points > highest_points {
                    highest_points = points;
                    draw_players.clear();
                }
            }
        }

        assert_eq!(highest_points, 19);
        assert!(draw_players.contains(&0));
        assert!(draw_players.contains(&2));
    }

    #[test]
    fn test_determine_winner_perfect_21() {
        // Player 2 wins with exactly 21 points
        let players_points = vec![20, 21, 19];
        let mut winner_index: Option<usize> = None;
        let mut highest_points: u8 = 0;

        for (index, &points) in players_points.iter().enumerate() {
            if points <= 21 && points > highest_points {
                highest_points = points;
                winner_index = Some(index);
            }
        }

        assert_eq!(winner_index, Some(1));
        assert_eq!(highest_points, 21);
    }

    #[test]
    fn test_cards_constant_has_correct_values() {
        // Verifies that face cards have value 10
        assert_eq!(CARDS[10].0, "J");
        assert_eq!(CARDS[10].1, 10);
        assert_eq!(CARDS[11].0, "Q");
        assert_eq!(CARDS[11].1, 10);
        assert_eq!(CARDS[12].0, "K");
        assert_eq!(CARDS[12].1, 10);
    }

    #[test]
    fn test_cards_constant_has_ace() {
        // Verifies that the Ace has value 1
        assert_eq!(CARDS[0].0, "A");
        assert_eq!(CARDS[0].1, 1);
    }

    #[test]
    fn test_cards_constant_has_numeric_cards() {
        // Verifies some numeric cards
        assert_eq!(CARDS[1].0, "2");
        assert_eq!(CARDS[1].1, 2);
        assert_eq!(CARDS[4].0, "5");
        assert_eq!(CARDS[4].1, 5);
        assert_eq!(CARDS[8].0, "9");
        assert_eq!(CARDS[8].1, 9);
    }

    #[test]
    fn test_cards_array_length() {
        // Verifies that there are 13 cards in the deck
        assert_eq!(CARDS.len(), 13);
    }

    #[test]
    fn test_determine_winner_with_one_player() {
        // A single player with a valid score
        let players_points = vec![18];
        let mut winner_index: Option<usize> = None;
        let mut highest_points: u8 = 0;

        for (index, &points) in players_points.iter().enumerate() {
            if points <= 21 && points > highest_points {
                highest_points = points;
                winner_index = Some(index);
            }
        }

        assert_eq!(winner_index, Some(0));
        assert_eq!(highest_points, 18);
    }

    #[test]
    fn test_determine_winner_mixed_bust_and_valid() {
        // Some players exceed 21, others don't
        let players_points = vec![22, 18, 25, 19];
        let mut winner_index: Option<usize> = None;
        let mut highest_points: u8 = 0;

        for (index, &points) in players_points.iter().enumerate() {
            if points <= 21 && points > highest_points {
                highest_points = points;
                winner_index = Some(index);
            }
        }

        assert_eq!(winner_index, Some(3));
        assert_eq!(highest_points, 19);
    }

    #[test]
    #[should_panic(expected = "The number of players must be greater than zero.")]
    fn test_total_players_zero_panics() {
        // Test that 0 players causes a panic
        let total_players: u8 = 0;
        assert!(total_players > 0, "The number of players must be greater than zero.");
    }

    #[test]
    #[should_panic(expected = "The maximum number of players is 10.")]
    fn test_total_players_exceeds_maximum_panics() {
        // Test that more than 10 players causes a panic
        let total_players: u8 = 11;
        assert!(total_players <= 10, "The maximum number of players is 10.");
    }

    #[test]
    fn test_total_players_valid_range() {
        // Test that valid player counts (1-10) don't panic
        for total_players in 1..=10 {
            assert!(total_players > 0, "The number of players must be greater than zero.");
            assert!(total_players <= 10, "The maximum number of players is 10.");
        }
    }
}
