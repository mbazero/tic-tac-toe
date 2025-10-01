use anyhow::Result;
use anyhow::anyhow;
use std::hash::Hash;
use std::ops::Index;
use std::ops::IndexMut;
use std::path::Path;

use ordered_float::OrderedFloat;

use crate::game::GameStateRef;
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
    q_table: QTable,
}

impl QTableMoveStrategy {
    pub fn new(q_table: QTable) -> Self {
        Self { q_table }
    }
}

impl MoveStrategy for QTableMoveStrategy {
    fn get_move<B: Board>(
        &mut self,
        GameStateRef {
            cur_player, board, ..
        }: GameStateRef<'_, B>,
    ) -> BoardIdx {
        let state = State(cur_player, board.as_bitset_board());
        self.q_table
            .max_action(state)
            .expect("no max action found")
            .0
            .0
    }
}

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct State(PlayerId, BitsetBoard);

impl State {
    const CARDINALITY: usize = 1 << 19;
}

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Action(BoardIdx);

impl Action {
    const CARDINALITY: usize = 9;
}

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct StateAction(usize);

#[allow(dead_code)]
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

    fn board(self) -> BitsetBoard {
        BitsetBoard {
            x_positions: self.x_positions() as u16,
            o_positions: self.o_positions() as u16,
        }
    }
}

type Reward = OrderedFloat<f64>;
type QValue = OrderedFloat<f64>;

#[derive(Clone, Debug)]
pub struct QTable(Box<[QValue; StateAction::CARDINALITY]>);

impl QTable {
    pub const DEFAULT_FILE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/artifacts/q_table");

    pub fn max_q(&self, state: State) -> Option<QValue> {
        state
            .1
            .iter_available()
            .map(|i| self[StateAction::new(state, Action(i))])
            .max()
    }

    pub fn max_action(&self, state: State) -> Option<(Action, QValue)> {
        fn action_rank(Action(i): Action) -> u8 {
            match i {
                4 => 2,             // center
                0 | 2 | 6 | 8 => 1, // corners
                _ => 0,
            }
        }

        state
            .1
            .iter_available()
            .map(|i| (Action(i), self[StateAction::new(state, Action(i))]))
            .max_by(|(a1, q1), (a2, q2)| {
                q1.cmp(q2)
                    .then_with(|| action_rank(*a1).cmp(&action_rank(*a2)))
            })
    }

    pub fn write_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let bytes = bytemuck::cast_slice(self.0.as_slice());
        std::fs::write(path, bytes)?;
        Ok(())
    }

    pub fn read_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let bytes = std::fs::read(path)?;
        let data = bytemuck::cast_slice(&bytes)
            .to_vec()
            .try_into()
            .map_err(|_| anyhow!("Failed to cast q-table bytes"))?;
        Ok(Self(data))
    }
}

impl Index<StateAction> for QTable {
    type Output = QValue;

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
        let table = vec![QValue::default(); StateAction::CARDINALITY]
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
        let board = BitsetBoard::with_positions([0, 4, 7], [1, 3]).unwrap();
        let player = PlayerId::O;
        let action = 5;

        let sa = StateAction::new(State(player, board), Action(action));

        assert_eq!(board.x_positions as usize, sa.x_positions());
        assert_eq!(board.o_positions as usize, sa.o_positions());
        assert_eq!(player, sa.player_id());
        assert_eq!(action, sa.action().0);
    }
}
