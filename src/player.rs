use enum_dispatch::enum_dispatch;
use strum::Display;

use crate::{
    board::{Board, BoardIdx},
    player::{human::HumanMoveStrategy, random::RandomMoveStrategy},
};

pub mod human;
pub mod random;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Display)]
pub enum PlayerId {
    X,
    O,
}

impl PlayerId {
    pub fn other(self) -> Self {
        match self {
            PlayerId::X => PlayerId::O,
            PlayerId::O => PlayerId::X,
        }
    }
}

#[enum_dispatch]
pub enum MoveStrategyEnum {
    HumanMoveStrategy,
    RandomMoveStrategy,
}

#[enum_dispatch(MoveStrategyEnum)]
pub trait MoveStrategy {
    fn get_move(&mut self, board: &impl Board) -> BoardIdx;
}
