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

impl Default for RandomMoveStrategy {
    fn default() -> Self {
        Self {
            rng: SmallRng::from_os_rng(),
        }
    }
}

impl RandomMoveStrategy {
    pub fn from_seed(seed: u64) -> Self {
        Self {
            rng: SmallRng::seed_from_u64(seed),
        }
    }
}

impl MoveStrategy for RandomMoveStrategy {
    fn get_move<B: Board>(&mut self, GameStateRef { board, .. }: GameStateRef<'_, B>) -> BoardIdx {
        *board.available().choose(&mut self.rng).unwrap()
    }
}
