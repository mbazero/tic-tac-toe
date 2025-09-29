use std::fmt::Display;

use crate::{board::Board, player::PlayerId};

use super::BoardIdx;

macro_rules! bitmask {
    ($($idx:expr),* $(,)?) => {{
        let mut m: u16 = 0;
        $(
            m |= 1u16 << $idx;
        )*
        m
    }};
}

#[derive(Copy, Clone, Debug, Default, Hash, Eq, PartialEq)]
pub struct BitsetBoard {
    pub x_positions: u16,
    pub o_positions: u16,
}

impl BitsetBoard {
    const WIN_MASKS: [u16; 8] = [
        // Rows
        bitmask![0, 1, 2],
        bitmask![3, 4, 5],
        bitmask![6, 7, 8],
        // Columns
        bitmask![0, 3, 6],
        bitmask![1, 4, 7],
        bitmask![2, 5, 8],
        // Diagonals
        bitmask![0, 4, 8],
        bitmask![2, 4, 6],
    ];

    fn positions(&self, player: PlayerId) -> u16 {
        match player {
            PlayerId::X => self.x_positions,
            PlayerId::O => self.o_positions,
        }
    }

    fn positions_mut(&mut self, player: PlayerId) -> &mut u16 {
        match player {
            PlayerId::X => &mut self.x_positions,
            PlayerId::O => &mut self.o_positions,
        }
    }
}

impl Board for BitsetBoard {
    fn set_unchecked(&mut self, idx: BoardIdx, player: PlayerId) {
        *self.positions_mut(player) |= 1 << idx;
    }

    fn get_unchecked(&self, idx: BoardIdx) -> Option<PlayerId> {
        let mask = 1 << idx;
        if self.x_positions & mask != 0 {
            Some(PlayerId::X)
        } else if self.o_positions & mask != 0 {
            Some(PlayerId::O)
        } else {
            None
        }
    }

    fn is_winner(&self, player: PlayerId) -> bool {
        let moves = self.positions(player);
        Self::WIN_MASKS.into_iter().any(|mask| mask & moves == mask)
    }

    fn iter(&self) -> impl Iterator<Item = Option<PlayerId>> {
        (0u8..9).map(|i| self.get_unchecked(i))
    }

    fn reset(&mut self) {
        *self = Self::default();
    }

    fn check_set(&self, idx: BoardIdx) -> Result<(), super::SetError> {
        if self.get(idx)?.is_some() {
            return Err(super::SetError::CellOccupied);
        }
        Ok(())
    }
}

impl Display for BitsetBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.display(f)
    }
}
