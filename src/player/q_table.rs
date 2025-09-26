use std::collections::HashMap;
use std::hash::Hash;

use crate::{
    board::{Board, BoardIdx, bitset::BitsetBoard},
    player::{MoveStrategy, PlayerId},
};

pub mod train;

#[derive(Default)]
pub struct QTableMoveStrategy {
    player_id: PlayerId,
    q_table: QTable<State, Action, Reward>,
}

impl MoveStrategy for QTableMoveStrategy {
    fn get_move(&mut self, board: &impl Board) -> BoardIdx {
        todo!()
    }

    fn reset(&mut self) {}
}

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct State(BitsetBoard);

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct Action {
    move_idx: BoardIdx,
}

#[derive(Default, Debug, Copy, Clone, PartialEq)]
struct Reward(f64);

#[derive(Default, Clone, Debug)]
struct QTable<S, A, R> {
    default_reward: R,
    q_map: HashMap<(S, A), R>,
}

impl<S: Hash + Eq, A: Hash + Eq, R: Copy> QTable<S, A, R> {
    fn q_value(&self, state: S, action: A) -> R {
        self.q_map
            .get(&(state, action))
            .copied()
            .unwrap_or(self.default_reward)
    }
}
