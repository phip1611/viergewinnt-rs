use crate::{Gameboard, Player};
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;

/// minmax step of updating the board and getting the current score for the current player.
/// Recursively calls the minmax function [`_check_move_recursive_minmax`] again.
fn minmax_step<const W: usize, const H: usize>(
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
    let fn_insert_update_recursive = |gameboard: &Gameboard<W, H>, col: usize| {
        let mut gameboard_clone = gameboard.clone();
        gameboard_clone
            .insert_player_chip(col, current_player)
            .unwrap();

        // skip col here, we take the col from the top level
        let (_, score) =
            _check_move_recursive_minmax(gameboard_clone, target_player, next_player, depth + 1);
        (col, score)
    };

    // top level: parallelize work
    if depth == 0 {
        let reduced = gameboard
            .available_columns_iter()
            // rayon wants an owned collection
            .collect::<Vec<_>>()
            .into_par_iter()
            .map(|col| fn_insert_update_recursive(gameboard, col))
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
            let (_, score) = fn_insert_update_recursive(gameboard, col);

            if better_score(score, best_score) {
                best_score = score;
                best_col = Some(col);
            }
        }
    }

    (best_col, best_score)
}

/// Max depth, determined experimentally. Single-threaded 8, multi-threaded 9.
const MAX_DEPTH: usize = 9;

/// Recursively minmax logic including the recursion end conditions and evaluation of
/// winning/losing moves.
fn _check_move_recursive_minmax<const W: usize, const H: usize>(
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
            return (None /* upper level knows col */, 10 - depth as i32);
        }
        // Opponent wins
        else if target_player != current_player && gameboard.check_for_winner(current_player) {
            // späte Niederlage "weniger schlimm"
            return (None /* upper level knows col */, -10 + depth as i32);
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
        minmax_step(
            &gameboard,
            target_player,
            current_player,
            current_player.opponent(),
            depth,
            i32::MIN,
            &|new, best| new > best,
        )
    } else {
        minmax_step(
            &gameboard,
            target_player,
            current_player,
            current_player.opponent(),
            depth,
            i32::MAX,
            &|new, best| new < best,
        )
    }
}

pub fn check_best_move_recursive_minmax<const W: usize, const H: usize>(
    gameboard: Gameboard<W, H>,
    current_player: Player,
) -> (
    Option<usize>, /* move: col */
    i32,           /* score: pos: moves leading to win, neg: moves leading to loss */
) {
    _check_move_recursive_minmax(gameboard, current_player, current_player, 0)
}

#[cfg(test)]
mod tests {}
