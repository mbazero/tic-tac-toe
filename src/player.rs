use enum_dispatch::enum_dispatch;
use strum::Display;

use crate::{
    board::{Board, BoardIdx},
    game::GameStateRef,
    player::{human::HumanMoveStrategy, random::RandomMoveStrategy},
};

pub mod human;
pub mod q_table;
pub mod random;

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Display, Hash)]
pub enum PlayerId {
    #[default]
    X = 0,
    O = 1,
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
    fn get_move<B: Board>(&mut self, game_state: GameStateRef<'_, B>) -> BoardIdx;
}
