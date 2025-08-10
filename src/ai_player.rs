use crate::{Game, Player};

#[must_use]
pub fn search_best_move<const W: usize, const H: usize>(
    game: &Game<W, H>,
    player: Player,
) -> usize /* column */ {
    // Optimization: Take middle when not taken yet
    if game.round() < 2 {
        let middle = game.board().width() / 2;
        if game.board().free_slots_in_column(middle) == game.board().height() {
            return middle;
        }
    }

    super::minmax::minmax_search::<W, H>(game.board().clone(), player)
}
