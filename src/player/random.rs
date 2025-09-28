use rand::{rngs::ThreadRng, seq::IndexedRandom};

use crate::{
    board::{Board, BoardIdx},
    player::MoveStrategy,
};

#[derive(Default)]
pub struct RandomMoveStrategy {
    rng: ThreadRng,
}

impl MoveStrategy for RandomMoveStrategy {
    fn get_move(&mut self, board: &impl Board) -> BoardIdx {
        let open_squares: Vec<_> = (0u8..)
            .zip(board.iter())
            .filter_map(|(i, x)| x.is_none().then_some(i))
            .collect();
        *open_squares.choose(&mut self.rng).unwrap()
    }
}
