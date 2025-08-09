//! Basic game logic of Vier gewinnt in Rust.

#![no_std]
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

extern crate alloc;

pub mod minmax;

use core::error::Error;
use core::fmt::{Debug, Formatter};
use core::{cmp, fmt};

/// Number of coins in a row to win the game.
const SERIES_LEN: usize = 4;

/// The new state after a coin was inserted into the board.
#[derive(Debug, PartialOrd, PartialEq, Clone, Copy, Eq)]
pub enum NewGameboardState {
    /// Game continues.
    Normal,
    /// Player won with that insertion.
    PlayerWon(Player),
    /// The game field is full without a winner.
    Draft,
}

#[derive(Debug, PartialOrd, PartialEq, Clone, Copy, Eq)]
pub enum GameboardError {
    /// Column is full.
    ColumnFull,
    InvalidColumn,
}

impl fmt::Display for GameboardError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        Debug::fmt(self, f)
    }
}

impl Error for GameboardError {}

#[derive(Debug, PartialOrd, PartialEq, Clone, Eq)]
pub struct Gameboard<const W: usize = 7, const H: usize = 6> {
    /// Board: rows --> col --> field
    /// Technical indices correspond to the logical indices:
    ///   (row=0,col=0) <==> bottom left of game board
    pub board: [[Option<Player>; W]; H],
}

impl<const W: usize, const H: usize> Default for Gameboard<W, H> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const W: usize, const H: usize> Gameboard<W, H> {
    #[must_use]
    pub fn new() -> Self {
        assert!(W >= SERIES_LEN);
        assert!(H >= SERIES_LEN);

        let board = [[None; W]; H];
        Self { board }
    }

    /// Returns the index to the next free slot in the selected column.
    ///
    /// Returns `None` if there are no more free slots.
    fn next_slot_in_column(&self, column_index: usize) -> Option<usize> {
        (0..H).find(|&row_index| self.board[row_index][column_index].is_none())
    }

    /// Emits the column indices where moves are legal.
    pub fn available_columns_iter(&self) -> impl Iterator<Item = usize> {
        (0..W).filter(|&col| self.next_slot_in_column(col).is_some())
    }

    /// Returns the number of free slots in the given column.
    #[must_use]
    pub fn free_slots_in_column(&self, column: usize) -> usize {
        for row in 0..H {
            if self.board[row][column].is_none() {
                return H - row;
            }
        }
        0
    }

    /// Returns the number of free slots in total.
    #[must_use]
    pub fn free_slots_in_total(&self) -> usize {
        (0..W).map(|col| self.free_slots_in_column(col)).sum()
    }

    /// Returns whether the game is over, i.e., there are no legal moves.
    #[must_use]
    pub fn gameover(&self) -> bool {
        self.available_columns_iter().count() == 0
    }

    pub fn insert_player_chip(
        &mut self,
        column_index: usize,
        player: Player,
    ) -> Result<(), GameboardError> {
        if column_index >= W {
            return Err(GameboardError::InvalidColumn);
        }

        let row_index = self
            .next_slot_in_column(column_index)
            .ok_or(GameboardError::ColumnFull)?;
        self.board[row_index][column_index] = Some(player);
        Ok(())
    }

    fn check_for_winner_vertically(&self, player: Player) -> bool {
        // check vertically
        for col in 0..W {
            let rows_to_check = H - SERIES_LEN + 1;
            for row in 0..rows_to_check {
                if self.board[row][col] == Some(player)
                    && self.board[row + 1][col] == Some(player)
                    && self.board[row + 2][col] == Some(player)
                    && self.board[row + 3][col] == Some(player)
                {
                    return true;
                }
            }
        }

        false
    }

    fn check_for_winner_horizontally(&self, player: Player) -> bool {
        // check vertically
        for row in 0..H {
            let cols_to_check = W - SERIES_LEN + 1;
            for col in 0..cols_to_check {
                if self.board[row][col] == Some(player)
                    && self.board[row][col + 1] == Some(player)
                    && self.board[row][col + 2] == Some(player)
                    && self.board[row][col + 3] == Some(player)
                {
                    return true;
                }
            }
        }

        false
    }

    fn check_for_winner_diagonally(&self, player: Player) -> bool {
        // check diagonally (`/`):
        // -> iteration top-left to bottom-right
        // -> diagonals going from bottom-left to top-right
        {
            // skip unneeded diagonals
            let d_min = SERIES_LEN - 1;
            let d_max = H + W - SERIES_LEN - 1;

            for d in d_min..=d_max {
                let row_begin = d.saturating_sub(W - 1);
                let row_end = if d < H { d } else { H - 1 };

                let d_len = row_end - row_begin + 1;

                // Should not happen due to precondition
                debug_assert!(d_len >= SERIES_LEN);

                // We iterate only as far as we can find a valid series
                for row in row_begin..=(row_end + 1 - SERIES_LEN) {
                    let col = d - row;

                    if self.board[row][col] == Some(player)
                        && self.board[row + 1][col - 1] == Some(player)
                        && self.board[row + 2][col - 2] == Some(player)
                        && self.board[row + 3][col - 3] == Some(player)
                    {
                        return true;
                    }
                }
            }
        }

        // check diagonally (`\`):
        // -> iteration bottom-left to top-right
        // -> diagonals going from bottom-left to top-right
        {
            // skip unneeded diagonals
            let d_min = -(W as isize - 1);
            let d_low = d_min.max(4 - W as isize);
            let d_high = H as isize - SERIES_LEN as isize;

            for d in d_low..=d_high {
                // r in [max(0,k), min(H, W+k))
                let row_begin = if d >= 0 { d as usize } else { 0 };
                // W + d might be negative if d < -W, but we filtered that out by d_low
                let row_end_exclusive = cmp::min(H, (W as isize + d) as usize);

                let d_len = row_end_exclusive.saturating_sub(row_begin);
                // Should not happen due to precondition
                debug_assert!(d_len >= SERIES_LEN);

                // We iterate only as far as we can find a valid series
                for row in row_begin..(row_end_exclusive + 1 - SERIES_LEN) {
                    let col = (row as isize - d) as usize;

                    if self.board[row][col] == Some(player)
                        && self.board[row + 1][col + 1] == Some(player)
                        && self.board[row + 2][col + 2] == Some(player)
                        && self.board[row + 3][col + 3] == Some(player)
                    {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Check if there is a winner.
    #[must_use]
    pub fn check_for_winner(&self, player: Player) -> bool {
        self.check_for_winner_horizontally(player)
            || self.check_for_winner_vertically(player)
            || self.check_for_winner_diagonally(player)
    }

    #[must_use]
    pub const fn width(&self) -> usize {
        W
    }

    #[must_use]
    pub const fn height(&self) -> usize {
        H
    }
}

#[derive(Copy, Clone, PartialOrd, PartialEq, Eq, Debug)]
pub enum Player {
    Player1,
    Player2,
}

impl Player {
    #[must_use]
    pub fn opponent(self) -> Self {
        if self == Self::Player1 {
            Self::Player2
        } else {
            Self::Player1
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use crate::{Gameboard, Player};
    use std::vec::Vec;

    #[test]
    fn test_next_slot_in_column() {
        let mut board = Gameboard::<7, 6>::new();
        assert_eq!(board.next_slot_in_column(0), Some(0));

        for i in 0..board.height() - 1 {
            board.board[i][0] = Some(Player::Player1);
            assert_eq!(board.next_slot_in_column(0), Some(i + 1));
        }

        board.board[board.height() - 1][0] = Some(Player::Player1);
        assert_eq!(board.next_slot_in_column(0), None);
    }

    #[test]
    fn test_free_slots_in_column() {
        let mut board = Gameboard::<7, 6>::new();
        assert_eq!(board.free_slots_in_column(0), 6);

        for i in 0..board.height() {
            board.board[i][0] = Some(Player::Player1);
            assert_eq!(board.free_slots_in_column(0), 6 - i - 1);
        }

        assert_eq!(board.free_slots_in_column(0), 0);
    }

    #[test]
    fn test_free_slots_in_total() {
        let mut board = Gameboard::<7, 6>::new();
        let total_slots = board.width() * board.height();
        assert_eq!(board.free_slots_in_total(), total_slots);

        let mut counter = total_slots;
        for col in 0..board.width() {
            for row in 0..board.height() {
                board.board[row][col] = Some(Player::Player1);

                counter -= 1;
                assert_eq!(counter, board.free_slots_in_total());
            }
        }

        assert_eq!(board.free_slots_in_total(), 0);
    }

    #[test]
    fn find_winner_horizontally() {
        {
            let mut board = Gameboard::<7, 6>::new();
            board.board[4][0] = Some(Player::Player1);
            board.board[4][1] = Some(Player::Player1);
            board.board[4][2] = Some(Player::Player1);

            assert!(!board.check_for_winner_horizontally(Player::Player1));
            assert!(!board.check_for_winner(Player::Player1));
            assert!(!board.check_for_winner_horizontally(Player::Player2));
            assert!(!board.check_for_winner(Player::Player2));

            board.board[4][3] = Some(Player::Player1);
            assert!(board.check_for_winner_horizontally(Player::Player1));
            assert!(board.check_for_winner(Player::Player1));
            assert!(!board.check_for_winner_horizontally(Player::Player2));
            assert!(!board.check_for_winner(Player::Player2));
        }
        {
            let mut board = Gameboard::<7, 6>::new();
            board.insert_player_chip(0, Player::Player1).unwrap();
            board.insert_player_chip(1, Player::Player1).unwrap();
            board.insert_player_chip(2, Player::Player1).unwrap();
            board.insert_player_chip(3, Player::Player1).unwrap();
            assert!(board.check_for_winner_horizontally(Player::Player1));
            assert!(board.check_for_winner(Player::Player1));
            assert!(!board.check_for_winner_horizontally(Player::Player2));
            assert!(!board.check_for_winner(Player::Player2));
        }
        {
            let mut board = Gameboard::<4, 4>::new();
            board.insert_player_chip(0, Player::Player1).unwrap();
            board.insert_player_chip(1, Player::Player1).unwrap();
            board.insert_player_chip(2, Player::Player1).unwrap();
            board.insert_player_chip(3, Player::Player1).unwrap();
            assert!(board.check_for_winner_horizontally(Player::Player1));
            assert!(board.check_for_winner(Player::Player1));
            assert!(!board.check_for_winner_horizontally(Player::Player2));
            assert!(!board.check_for_winner(Player::Player2));
        }
    }

    #[test]
    fn find_winner_vertically() {
        {
            let mut board = Gameboard::<7, 6>::new();
            board.board[0][5] = Some(Player::Player1);
            board.board[1][5] = Some(Player::Player1);
            board.board[2][5] = Some(Player::Player1);

            assert!(!board.check_for_winner_vertically(Player::Player1));
            assert!(!board.check_for_winner(Player::Player1));
            assert!(!board.check_for_winner_vertically(Player::Player2));
            assert!(!board.check_for_winner(Player::Player2));

            board.board[3][5] = Some(Player::Player1);
            assert!(board.check_for_winner_vertically(Player::Player1));
            assert!(board.check_for_winner(Player::Player1));
            assert!(!board.check_for_winner_vertically(Player::Player2));
            assert!(!board.check_for_winner(Player::Player2));
        }
        {
            let mut board = Gameboard::<7, 6>::new();

            board.insert_player_chip(0, Player::Player1).unwrap();
            board.insert_player_chip(0, Player::Player1).unwrap();
            board.insert_player_chip(0, Player::Player1).unwrap();
            board.insert_player_chip(0, Player::Player1).unwrap();
            assert!(board.check_for_winner_vertically(Player::Player1));
            assert!(board.check_for_winner(Player::Player1));
            assert!(!board.check_for_winner_vertically(Player::Player2));
            assert!(!board.check_for_winner(Player::Player2));
        }
        {
            let mut board = Gameboard::<4, 4>::new();

            board.insert_player_chip(0, Player::Player1).unwrap();
            board.insert_player_chip(0, Player::Player1).unwrap();
            board.insert_player_chip(0, Player::Player1).unwrap();
            board.insert_player_chip(0, Player::Player1).unwrap();
            assert!(board.check_for_winner_vertically(Player::Player1));
            assert!(board.check_for_winner(Player::Player1));
            assert!(!board.check_for_winner_vertically(Player::Player2));
            assert!(!board.check_for_winner(Player::Player2));
        }
    }

    #[test]
    fn find_winner_diagonally() {
        // direction=\, winner=no
        {
            let mut board = Gameboard::<7, 6>::new();
            board.board[0][0] = Some(Player::Player1);
            board.board[1][1] = Some(Player::Player1);
            board.board[2][2] = Some(Player::Player1);

            assert!(!board.check_for_winner(Player::Player1));

            board.board[4][3] = Some(Player::Player1);
            assert!(!board.check_for_winner(Player::Player1));
        }
        // direction=\, winner=yes
        {
            let mut board = Gameboard::<7, 6>::new();
            board.board[0][0] = Some(Player::Player1);
            board.board[1][1] = Some(Player::Player1);
            board.board[2][2] = Some(Player::Player1);

            assert!(!board.check_for_winner(Player::Player1));

            board.board[3][3] = Some(Player::Player1);
            assert!(board.check_for_winner(Player::Player1));
        }
        // direction=\, winner=yes
        {
            let mut board = Gameboard::<7, 6>::new();
            board.board[0][3] = Some(Player::Player1);
            board.board[1][4] = Some(Player::Player1);
            board.board[2][5] = Some(Player::Player1);

            assert!(!board.check_for_winner(Player::Player1));

            board.board[3][6] = Some(Player::Player1);
            assert!(board.check_for_winner(Player::Player1));
        }
        // direction=/, winner=yes
        {
            let mut board = Gameboard::<7, 6>::new();
            board.board[4][3] = Some(Player::Player1);
            board.board[3][4] = Some(Player::Player1);
            board.board[2][5] = Some(Player::Player1);

            assert!(!board.check_for_winner(Player::Player1));

            board.board[1][6] = Some(Player::Player1);
            assert!(board.check_for_winner(Player::Player1));
        }
        // direction=\, winner=yes
        {
            let mut board = Gameboard::<4, 4>::new();
            board.board[0][0] = Some(Player::Player1);
            board.board[1][1] = Some(Player::Player1);
            board.board[2][2] = Some(Player::Player1);
            board.board[3][3] = Some(Player::Player1);
            assert!(board.check_for_winner_diagonally(Player::Player1));
            assert!(board.check_for_winner(Player::Player1));
        }
        // direction=/, winner=yes
        {
            let mut board = Gameboard::<4, 4>::new();
            board.board[0][3] = Some(Player::Player1);
            board.board[1][2] = Some(Player::Player1);
            board.board[2][1] = Some(Player::Player1);
            board.board[3][0] = Some(Player::Player1);
            assert!(board.check_for_winner_diagonally(Player::Player1));
            assert!(board.check_for_winner(Player::Player1));
        }
    }

    #[test]
    fn test_legal_moves_iter() {
        {
            let mut board = Gameboard::<7, 6>::new();

            fn fill_column(board: &mut Gameboard, col: usize) {
                for _ in 0..board.height() {
                    let _ = board.insert_player_chip(col, Player::Player1);
                }
            }

            assert_eq!(
                board
                    .available_columns_iter()
                    .collect::<Vec<_>>()
                    .as_slice(),
                &[0, 1, 2, 3, 4, 5, 6]
            );

            fill_column(&mut board, 1);
            fill_column(&mut board, 3);
            fill_column(&mut board, 5);
            fill_column(&mut board, 6);
            assert_eq!(
                board
                    .available_columns_iter()
                    .collect::<Vec<_>>()
                    .as_slice(),
                &[0, 2, 4]
            );

            fill_column(&mut board, 0);
            fill_column(&mut board, 2);
            fill_column(&mut board, 4);
            assert_eq!(
                board
                    .available_columns_iter()
                    .collect::<Vec<_>>()
                    .as_slice(),
                &[]
            );
            assert!(board.gameover());
        }
        {
            let mut board = Gameboard::<4, 4>::new();
            board.board[0][0] = Some(Player::Player2);
            board.board[0][1] = Some(Player::Player2);
            board.board[0][2] = Some(Player::Player1);
            board.board[0][3] = Some(Player::Player2);
            board.board[1][0] = Some(Player::Player1);
            board.board[1][1] = Some(Player::Player1);
            board.board[1][2] = Some(Player::Player2);
            board.board[1][3] = Some(Player::Player1);
            board.board[2][0] = Some(Player::Player2);
            board.board[2][1] = Some(Player::Player1);
            board.board[2][2] = Some(Player::Player1);
            board.board[2][3] = Some(Player::Player2);
            board.board[3][0] = Some(Player::Player1);
            board.board[3][1] = Some(Player::Player2);

            assert_eq!(
                board
                    .available_columns_iter()
                    .collect::<Vec<_>>()
                    .as_slice(),
                &[2, 3]
            );
            board.board[3][2] = Some(Player::Player1);
            board.board[3][3] = Some(Player::Player2);
            assert!(board.gameover());
        }
        {
            let mut board = Gameboard::<4, 4>::new();
            board.board[0][0] = Some(Player::Player2);
            board.board[0][2] = Some(Player::Player1);
            board.board[1][0] = Some(Player::Player2);
            board.board[1][2] = Some(Player::Player1);
            board.board[2][2] = Some(Player::Player1);
            board.board[3][2] = Some(Player::Player1);

            assert_eq!(
                board
                    .available_columns_iter()
                    .collect::<Vec<_>>()
                    .as_slice(),
                &[0, 1, 3]
            );
        }
    }
}
