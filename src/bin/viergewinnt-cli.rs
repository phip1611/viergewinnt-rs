#![deny(
    clippy::all,
    clippy::cargo,
    clippy::nursery,
    clippy::must_use_candidate,
    // clippy::restriction,
    // clippy::pedantic
)]
// now allow a few rules which are denied by the above statement
// --> they are ridiculous and not necessary
#![allow(
    clippy::suboptimal_flops,
    clippy::redundant_pub_crate,
    clippy::fallible_impl_from
)]
#![deny(missing_debug_implementations)]
#![deny(rustdoc::all)]

use viergewinnt_rs::{Game, Gameboard, Player, search_best_move};

fn print_board(board: &Gameboard) {
    // Print rows reverted to that it appears naturally.
    for row in board.board().iter().rev() {
        for col in row.iter() {
            let symbol = match col {
                None => ' ',
                Some(Player::Player1) => 'X',
                Some(Player::Player2) => 'O',
            };
            print!("{symbol},");
        }
        println!();
    }

    for _ in 0..board.width() {
        print!("--");
    }
    println!();

    for col_id in (0..board.width()).map(|i| i + 1) {
        print!("{col_id},");
    }
    println!();
}

fn main() {
    let mut game = Game::<7, 6>::new();
    let mut current_player = Player::Player1;

    println!("Let's play viergewinnt against the computer.");
    loop {
        println!("----------------");
        print_board(game.board());
        println!();

        if game.board().gameover() {
            println!("Gameover: draft");
            break;
        }

        // Human player
        if current_player == Player::Player1 {
            {
                print!("Choose your move (column): ");
                for col in game.board().available_columns_iter().map(|x| x + 1) {
                    print!("{col},");
                }
                println!();
            }

            let line = {
                let mut line = String::new();
                std::io::stdin().read_line(&mut line).unwrap();
                line.trim().parse::<usize>().unwrap()
            };
            // adapt to index
            let line = line - 1;

            game.insert_player_chip(line, current_player).unwrap();

            {
                if game.board().check_for_winner(current_player) {
                    println!("You won!");
                    break;
                }

                current_player = current_player.opponent();
            }
        }
        // Computer player
        else {
            // let best_move = board.legal_moves_iter().next().unwrap();
            let best_move = search_best_move::<7, 6>(&game, current_player);
            println!("Computer chose column {}", best_move + 1);
            game.insert_player_chip(best_move, current_player).unwrap();

            {
                if game.board().check_for_winner(current_player) {
                    println!("Computer won!");
                    break;
                }

                current_player = current_player.opponent();
            }
        }
    }

    println!("----------------");
    print_board(game.board());
}
