use viergewinnt_rs::minmax::minmax_search;
use viergewinnt_rs::{Gameboard, Player};

fn main() {
    let mut board = Gameboard::<7, 6>::new();
    let mut current_player = Player::Player1;

    println!("Let's play viergewinnt against the computer.");
    loop {
        println!("----------------");
        board.print();

        if board.gameover() {
            println!("Gameover: draft");
            break;
        }

        // Human player
        if current_player == Player::Player1 {
            {
                print!("Choose your move (column): ");
                for col in board.available_columns_iter() {
                    print!("{col},");
                }
                println!();
            }

            let line = {
                let mut line = String::new();
                std::io::stdin().read_line(&mut line).unwrap();
                line.trim().parse::<usize>().unwrap()
            };

            board.insert_player_chip(line, current_player).unwrap();

            {
                if board.check_for_winner(current_player) {
                    println!("You won!");
                    break;
                }

                current_player = current_player.opponent();
            }
        }
        // Computer player
        else {
            // let best_move = board.legal_moves_iter().next().unwrap();
            let (best_move, _) = minmax_search::<7, 6>(board.clone(), current_player);
            let best_move = best_move.expect("should have a possible move");
            board.insert_player_chip(best_move, current_player).unwrap();

            {
                if board.check_for_winner(current_player) {
                    println!("Computer won!");
                    break;
                }

                current_player = current_player.opponent();
            }
        }
    }

    println!("----------------");
    board.print();
}
