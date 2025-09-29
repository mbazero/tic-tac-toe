use std::cmp::Ordering;
use std::hash::Hash;
use std::ops::IndexMut;
use std::{collections::HashMap, ops::Index};

use crate::{
    board::{Board, BoardIdx, bitset::BitsetBoard},
    player::{MoveStrategy, PlayerId},
};

pub mod train;

trait AsBitsetBoard {
    fn as_bitset_board(&self) -> BitsetBoard;
}

impl<T: Board> AsBitsetBoard for &T {
    fn as_bitset_board(&self) -> BitsetBoard {
        let mut bitset_board = BitsetBoard::default();
        for i in 0..9 {
            if let Some(player) = self.get_unchecked(i) {
                bitset_board.set_unchecked(i, player);
            }
        }
        bitset_board
    }
}

// Use autoref specialization for efficient impl on BitsetBoard instances
impl AsBitsetBoard for BitsetBoard {
    fn as_bitset_board(&self) -> BitsetBoard {
        *self
    }
}

#[derive(Default)]
pub struct QTableMoveStrategy {
    player_id: PlayerId,
    q_table: QTable,
}

impl MoveStrategy for QTableMoveStrategy {
    fn get_move(&mut self, board: &impl Board) -> BoardIdx {
        let state = State(self.player_id, board.as_bitset_board());
        let (action, _) = self
            .q_table
            .action_max(state, Action::iter_available(board))
            .expect("no action found");
        action.0
    }
}

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct State(PlayerId, BitsetBoard);

impl State {
    const CARDINALITY: usize = 1 << 19;
}

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct Action(BoardIdx);

impl Action {
    const CARDINALITY: usize = 9;

    fn iter() -> impl Iterator<Item = Action> {
        (0..9).map(Action)
    }

    fn iter_available(board: &impl Board) -> impl Iterator<Item = Action> {
        board
            .iter()
            .zip(0..9)
            .filter_map(|(opt, i)| opt.is_none().then_some(Action(i)))
    }
}

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct StateAction(usize);

impl StateAction {
    const CARDINALITY: usize = State::CARDINALITY * Action::CARDINALITY;
    const ACTION_OFFSET: usize = 19;
    const PLAYER_ID_OFFSET: usize = 18;
    const X_POSITIONS_OFFSET: usize = 9;
    const O_POSITIONS_OFFSET: usize = 0;
    const ACTION_MASK: usize = 0x000F;
    const PLAYER_ID_MASK: usize = 0x0001;
    const POSITIONS_MASK: usize = 0x01FF;

    fn new(state: State, action: Action) -> Self {
        Self(
            (action.0 as usize) << Self::ACTION_OFFSET
                | (state.0 as usize) << Self::PLAYER_ID_OFFSET
                | (state.1.x_positions as usize) << Self::X_POSITIONS_OFFSET
                | (state.1.o_positions as usize) << Self::O_POSITIONS_OFFSET,
        )
    }

    fn action(self) -> Action {
        Action((self.0 >> Self::ACTION_OFFSET & Self::ACTION_MASK) as u8)
    }

    fn player_id(self) -> PlayerId {
        if self.0 >> Self::PLAYER_ID_OFFSET & Self::PLAYER_ID_MASK == 0 {
            PlayerId::X
        } else {
            PlayerId::O
        }
    }

    fn x_positions(self) -> usize {
        self.0 >> Self::X_POSITIONS_OFFSET & Self::POSITIONS_MASK
    }

    fn o_positions(self) -> usize {
        self.0 >> Self::O_POSITIONS_OFFSET & Self::POSITIONS_MASK
    }
}

#[derive(Clone, Debug)]
struct QTable(Box<[f64; StateAction::CARDINALITY]>);

impl QTable {
    fn action_max(
        &self,
        state: State,
        actions: impl IntoIterator<Item = Action>,
    ) -> Option<(Action, f64)> {
        actions
            .into_iter()
            .map(|action| (action, self[StateAction::new(state, action)]))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal))
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
        let table = vec![0.0; StateAction::CARDINALITY]
            .into_boxed_slice()
            .try_into()
            .unwrap();
        Self(table)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        board::{Board, bitset::BitsetBoard},
        player::{
            PlayerId,
            q_table::{Action, State, StateAction},
        },
    };

    #[test]
    fn test_state_action_packing() {
        let board = BitsetBoard::with_positions([0, 4, 7], [1, 3]);
        let player = PlayerId::O;
        let action = 5;

        let sa = StateAction::new(State(player, board), Action(action));

        assert_eq!(board.x_positions as usize, sa.x_positions());
        assert_eq!(board.o_positions as usize, sa.o_positions());
        assert_eq!(player, sa.player_id());
        assert_eq!(action, sa.action().0);
    }
}
