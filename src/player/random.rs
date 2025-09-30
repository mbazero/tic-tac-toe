use rand::{SeedableRng, rngs::SmallRng, seq::IndexedRandom};

use crate::{
    board::{Board, BoardIdx},
    game::GameStateRef,
    player::MoveStrategy,
};

#[derive(Clone)]
pub struct RandomMoveStrategy {
    rng: SmallRng,
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
        let open_squares: Vec<_> = (0u8..)
            .zip(board.iter())
            .filter_map(|(i, x)| x.is_none().then_some(i))
            .collect();
        *open_squares.choose(&mut self.rng).unwrap()
    }
}
