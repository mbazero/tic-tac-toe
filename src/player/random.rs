use rand::{SeedableRng, rngs::SmallRng, seq::IndexedRandom};

use crate::{
    board::{Board, BoardIdx},
    game::GameStateRef,
    player::MoveStrategy,
};

#[derive(Clone)]
pub struct RandomMoveStrategy {
    pub rng: SmallRng,
}

impl RandomMoveStrategy {
    pub fn new(rng: SmallRng) -> Self {
        Self { rng }
    }
}

impl MoveStrategy for RandomMoveStrategy {
    fn get_move<B: Board>(&mut self, GameStateRef { board, .. }: GameStateRef<'_, B>) -> BoardIdx {
        *board.available().choose(&mut self.rng).unwrap()
    }
}

impl Default for RandomMoveStrategy {
    fn default() -> Self {
        Self {
            rng: SmallRng::from_os_rng(),
        }
    }
}
