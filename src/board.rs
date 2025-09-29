use std::fmt::Display;

use thiserror::Error;

use crate::player::PlayerId;

pub mod array;
pub mod bitset;

pub type BoardIdx = u8;

#[derive(Error, Debug, Copy, Clone)]
pub enum SetError {
    #[error("cell is out of bounds")]
    CellOutOfBounds,
    #[error("cell is occupied")]
    CellOccupied,
}

impl From<GetError> for SetError {
    fn from(value: GetError) -> Self {
        match value {
            GetError::CellOutOfBounds => SetError::CellOutOfBounds,
        }
    }
}

#[derive(Error, Debug, Copy, Clone)]
pub enum GetError {
    #[error("cell is out of bounds")]
    CellOutOfBounds,
}

pub trait Board: Default + Display {
    fn check_set(&self, idx: BoardIdx) -> Result<(), SetError> {
        if self.get(idx)?.is_some() {
            return Err(SetError::CellOccupied);
        }
        Ok(())
    }

    fn set(&mut self, idx: BoardIdx, player: PlayerId) -> Result<(), SetError> {
        self.check_set(idx)?;
        self.set_unchecked(idx, player);
        Ok(())
    }

    fn get(&self, idx: BoardIdx) -> Result<Option<PlayerId>, GetError> {
        if idx >= 9 {
            return Err(GetError::CellOutOfBounds);
        }
        Ok(self.get_unchecked(idx))
    }

    fn set_unchecked(&mut self, idx: BoardIdx, player: PlayerId);

    fn get_unchecked(&self, idx: BoardIdx) -> Option<PlayerId>;

    fn is_winner(&self, player: PlayerId) -> bool;

    fn iter(&self) -> impl Iterator<Item = Option<PlayerId>>;

    fn reset(&mut self);

    fn with_positions(
        x_positions: impl IntoIterator<Item = BoardIdx>,
        o_positions: impl IntoIterator<Item = BoardIdx>,
    ) -> Result<Self, SetError> {
        let mut board = Self::default();
        for x_pos in x_positions {
            board.set(x_pos, PlayerId::X)?;
        }
        for o_pos in o_positions {
            board.set(o_pos, PlayerId::O)?;
        }
        Ok(board)
    }

    fn display(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cc = |i: u8| match self.get_unchecked(i) {
            Some(PlayerId::X) => 'X',
            Some(PlayerId::O) => 'O',
            None => ' ',
        };
        writeln!(f, "{}|{}|{}", cc(0), cc(1), cc(2))?;
        writeln!(f, "-----")?;
        writeln!(f, "{}|{}|{}", cc(3), cc(4), cc(5))?;
        writeln!(f, "-----")?;
        write!(f, "{}|{}|{}", cc(6), cc(7), cc(8))?;
        Ok(())
    }
}
