//! MinMax algorithmic search for Vier gewinnt.

use crate::{Gameboard, Player};
use alloc::vec::Vec;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;

/// Max depth, determined experimentally.
///
/// On my machines for a 7x6 board:
/// - Single-threaded: 8
/// - Multi-threaded: 9
pub const MAX_DEPTH: usize = 10;

/// Inclusive cutoff depth.
///
/// Assuming cols=7 and cutoff=2: 7 + 7² + 7³ == 7 + 49 + 343 == 699 Threads.
///
/// The Overhead of these many threads is negligible compared to the time it
/// takes to walk the search tree. This way, the program can easily use
/// hundreds of cors on big machines.
const PARALLEL_CUTOFF_DEPTH: usize = 2;

/// Searches for the best possible move for the current player at the given
/// search depth using the minimax algorithm, with optional parallelization at
/// the top search level.
///
/// This function evaluates all legal moves from the current game state by
/// simulating each move, updating the game board, and performing a recursive
/// minimax search via [`minmax_search_recursive`].
fn search_best_move_in_depth<const W: usize, const H: usize>(
    gameboard: &Gameboard<W, H>,
    target_player: Player,
    current_player: Player,
    next_player: Player,
    depth: usize,
    initial_score: i32,
    better_score: impl Fn(i32, i32) -> bool + 'static + Send + Sync,
) -> (Option<usize>, i32) {
    let mut best_score = initial_score;
    let mut best_col = None;

    debug_assert_ne!(gameboard.available_columns_iter().count(), 0);

    // Inserts the player coin, updates the field, and performs a recursive
    // search for following moves.
    let simulate_move = |gameboard: &Gameboard<W, H>, col: usize| {
        let mut gameboard_clone = gameboard.clone();
        gameboard_clone
            .insert_player_chip(col, current_player)
            .unwrap();

        // skip col here, we take the col from the top level
        let (_, score) =
            minmax_search_recursive(gameboard_clone, target_player, next_player, depth + 1);
        (col, score)
    };

    // top level: parallelize work
    if depth <= PARALLEL_CUTOFF_DEPTH {
        let reduced = gameboard
            .available_columns_iter()
            // rayon wants an owned collection
            .collect::<Vec<_>>()
            .into_par_iter()
            .map(|col| simulate_move(gameboard, col))
            .reduce(
                || (usize::MAX, initial_score),
                |acc, (col, score)| {
                    if better_score(score, acc.1) {
                        (col, score)
                    } else {
                        acc
                    }
                },
            );
        best_score = reduced.1;
        best_col = Some(reduced.0);
    }
    // Normal recursion
    else {
        for col in gameboard.available_columns_iter() {
            let (_, score) = simulate_move(gameboard, col);

            if better_score(score, best_score) {
                best_score = score;
                best_col = Some(col);
            }
        }
    }

    (best_col, best_score)
}


/// Should be more than MAX_DEPTH.
pub const SCORE_FACTOR: i32 = MAX_DEPTH as i32 + 1;

/// Recursive helper for [`minmax_search_recursive`].
fn minmax_search_recursive<const W: usize, const H: usize>(
    gameboard: Gameboard<W, H>,
    target_player: Player,
    current_player: Player,
    depth: usize,
) -> (
    Option<usize>, /* move: col */
    i32,           /* score: pos: moves leading to win, neg: moves leading to loss */
) {
    // We start with the recursion tail: Can we stop the recursion?
    {
        // Target player wins
        if target_player == current_player && gameboard.check_for_winner(target_player) {
            // schneller Sieg besser
            return (
                None, /* upper level knows col */
                SCORE_FACTOR - depth as i32,
            );
        }
        // Opponent wins
        else if target_player != current_player && gameboard.check_for_winner(current_player) {
            // späte Niederlage "weniger schlimm"
            return (
                None, /* upper level knows col */
                -SCORE_FACTOR + depth as i32,
            );
        }
        // draw
        else if gameboard.gameover() {
            return (None /* upper level knows col */, 0);
        }
    }

    // Abort. Too deep. Already takes quite some time with 7x6 fields..
    if depth > MAX_DEPTH {
        // TODO room for improvement: evaluate board, e.g., look for chains of three or so!
        return (None /* upper level knows col */, 0);
    }

    if current_player == target_player {
        search_best_move_in_depth(
            &gameboard,
            target_player,
            current_player,
            current_player.opponent(),
            depth,
            i32::MIN,
            |new, best| new > best,
        )
    } else {
        search_best_move_in_depth(
            &gameboard,
            target_player,
            current_player,
            current_player.opponent(),
            depth,
            i32::MAX,
            |new, best| new < best,
        )
    }
}

/// Performs a recursive MinMax search from the given board state.
///
/// At each step:
/// - Checks for terminal conditions (win, loss, draw) and assigns scores.
/// - Stops recursion at [`MAX_DEPTH`].
/// - Chooses the best move depending on whether the current player is
///   maximizing or minimizing the score.
#[must_use]
pub fn minmax_search<const W: usize, const H: usize>(
    gameboard: Gameboard<W, H>,
    current_player: Player,
) -> usize {
    minmax_search_recursive(gameboard, current_player, current_player, 0)
        .0
        .expect("should have legal move")
}

#[cfg(test)]
mod tests {
    use crate::minmax::minmax_search;
    use crate::{Gameboard, Player};

    #[test]
    fn test_minmax() {
        let mut board = Gameboard::<4, 4>::new();
        board.insert_player_chip(0, Player::Player1).unwrap();
        board.insert_player_chip(0, Player::Player2).unwrap();
        board.insert_player_chip(0, Player::Player1).unwrap();
        board.insert_player_chip(0, Player::Player2).unwrap();

        board.insert_player_chip(1, Player::Player1).unwrap();
        board.insert_player_chip(1, Player::Player2).unwrap();
        board.insert_player_chip(1, Player::Player1).unwrap();
        board.insert_player_chip(1, Player::Player2).unwrap();

        board.insert_player_chip(2, Player::Player1).unwrap();
        board.insert_player_chip(2, Player::Player1).unwrap();
        board.insert_player_chip(2, Player::Player1).unwrap();

        board.insert_player_chip(3, Player::Player2).unwrap();
        board.insert_player_chip(3, Player::Player2).unwrap();
        board.insert_player_chip(3, Player::Player2).unwrap();

        let best_move = minmax_search(board, Player::Player1);
        assert_eq!(best_move, 2);
    }
}
