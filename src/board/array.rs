use std::fmt::Display;

use crate::{board::Board, player::PlayerId};

use super::BoardIdx;

#[derive(Clone, Debug, Default)]
pub struct ArrayBoard([Option<PlayerId>; 9]);

#[allow(unused)]
impl ArrayBoard {
    const WIN_LANES: [[BoardIdx; 3]; 8] = [
        // Rows
        [0, 1, 2],
        [3, 4, 5],
        [6, 7, 8],
        // Columns
        [0, 3, 6],
        [1, 4, 7],
        [2, 5, 8],
        // Diagonals
        [0, 4, 8],
        [2, 4, 6],
    ];
}

impl Board for ArrayBoard {
    fn set_unchecked(&mut self, idx: BoardIdx, player: PlayerId) {
        self.0[idx as usize] = Some(player);
    }

    fn get_unchecked(&self, idx: BoardIdx) -> Option<PlayerId> {
        self.0[idx as usize]
    }

    fn is_winner(&self, player: PlayerId) -> bool {
        Self::WIN_LANES.into_iter().any(|lane| {
            lane.into_iter()
                .all(|i| self.get_unchecked(i) == Some(player))
        })
    }

    fn iter(&self) -> impl Iterator<Item = Option<PlayerId>> {
        self.0.iter().copied()
    }
}

impl Display for ArrayBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.display(f)
    }
}
