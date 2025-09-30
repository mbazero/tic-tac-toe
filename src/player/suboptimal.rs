use rand::seq::IndexedRandom;

use crate::{
    board::{Board, BoardIdx, BoardVec},
    game::GameStateRef,
    player::{MoveStrategy, random::RandomMoveStrategy},
};

// Move strategy that always makes a losing first move and otherwise plays randomly
#[derive(Default, Clone)]
pub struct SuboptimalMoveStrategy {
    random_strategy: RandomMoveStrategy,
}

impl SuboptimalMoveStrategy {
    const SUBOPTIMAL_FIRST_MOVES: [BoardIdx; 4] = [1, 3, 5, 7];

    pub fn new(random_strategy: RandomMoveStrategy) -> Self {
        Self { random_strategy }
    }
}

impl MoveStrategy for SuboptimalMoveStrategy {
    fn get_move<B: Board>(&mut self, game_state: GameStateRef<'_, B>) -> BoardIdx {
        if [0, 1].contains(&game_state.turns.len()) {
            let suboptimal_moves: BoardVec = game_state
                .board
                .iter_available()
                .filter(|i| Self::SUBOPTIMAL_FIRST_MOVES.contains(i))
                .collect();
            *suboptimal_moves
                .choose(&mut self.random_strategy.rng)
                .unwrap()
        } else {
            self.random_strategy.get_move(game_state)
        }
    }
}
