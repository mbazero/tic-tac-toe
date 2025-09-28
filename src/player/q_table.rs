use std::hash::Hash;
use std::ops::IndexMut;
use std::{collections::HashMap, ops::Index};

use crate::{
    board::{Board, BoardIdx, bitset::BitsetBoard},
    player::{MoveStrategy, PlayerId},
};

pub mod train;

#[derive(Default)]
pub struct QTableMoveStrategy {
    player_id: PlayerId,
    q_table: QTable,
}

impl MoveStrategy for QTableMoveStrategy {
    fn get_move(&mut self, board: &impl Board) -> BoardIdx {
        todo!()
    }
}

struct State {
    player_id: bool,
    x_positions: u16,
    o_positions: u16,
}

impl State {
    const CARDINALITY: usize = 1 << 19;
}

struct Action {
    move_idx: u8,
}

impl Action {
    const CARDINALITY: usize = 1 << 9;
}

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct StateAction(usize);

impl StateAction {
    // NOW: Fix this shit
    const NUM_BITS: usize = 19;
    const CARDINALITY: usize = 1 << StateAction::NUM_BITS;
    const POSITIONS_MASK: usize = 0x01FF;

    fn new(state: State, action: Action) -> Self {
        Self(
            ((action.move_idx as usize) << 19)
                & ((state.player_id as usize) << 18)
                & ((state.x_positions as usize) << 9)
                & (state.o_positions as usize),
        )
    }

    fn iter_actions(state: State) -> impl Iterator<Item = Self> {
        todo!();
        std::iter::empty()
    }

    fn player_id(self) -> PlayerId {
        if self.0 >> 18 == 0 {
            PlayerId::X
        } else {
            PlayerId::O
        }
    }

    fn player_x_positions(self) -> usize {
        self.0 & Self::POSITIONS_MASK
    }

    fn player_o_positions(self) -> usize {
        (self.0 >> 9) & Self::POSITIONS_MASK
    }
}

#[derive(Clone, Debug)]
struct QTable(Box<[f64; StateAction::CARDINALITY]>);

impl QTable {
    fn action_max(&self, state: State) -> f64 {
        StateAction::iter_actions(state)
            .map(|sa| self[sa])
            .reduce(f64::max)
            .expect("action max should exist")
    }
}

impl Index<StateAction> for QTable {
    type Output = f64;

    fn index(&self, index: StateAction) -> &Self::Output {
        &self.0[index.0]
    }
}

impl IndexMut<StateAction> for QTable {
    fn index_mut(&mut self, index: StateAction) -> &mut Self::Output {
        &mut self.0[index.0]
    }
}

impl Default for QTable {
    fn default() -> Self {
        Self(
            vec![0.0; StateAction::CARDINALITY]
                .into_boxed_slice()
                .try_into()
                .unwrap(),
        )
    }
}
