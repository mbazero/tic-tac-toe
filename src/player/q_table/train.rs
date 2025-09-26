use rand::{Rng, rngs::ThreadRng};

use crate::{
    board::{Board, BoardIdx, bitset::BitsetBoard},
    game::{Game, GameState},
    player::{
        MoveStrategy, PlayerId,
        q_table::{Action, QTable, QTableMoveStrategy, Reward, State},
        random::RandomMoveStrategy,
    },
};

struct Params {
    update: UpdateParams,
    training: TrainingParams,
    explore: ExploreParams,
}

struct UpdateParams {
    discount_factor: f64, // gamma
    learning_rate: f64,   // alpha
}

struct TrainingParams {
    num_episodes: u64,
    max_steps_per_episode: u64,
}

enum ExploreParams {
    FixedEpsilon {
        epsilon: f64,
    },
    EpsilonDecay {
        epsilon_start: f64,
        epsilon_end: f64,
        epsilon_decay: f64,
    },
}

impl ExploreParams {
    fn get_epsilon(&self, episode: u64) -> f64 {
        match self {
            ExploreParams::FixedEpsilon { epsilon } => *epsilon,
            ExploreParams::EpsilonDecay {
                epsilon_start,
                epsilon_end,
                epsilon_decay,
            } => (*epsilon_end).max(*epsilon_start - *epsilon_decay * episode as f64),
        }
    }
}

#[derive(Default)]
struct EpsilonGreedyMoveStrategy {
    rng: ThreadRng,
    epsilon: f64,
    q_table_strat: QTableMoveStrategy,
    random_strat: RandomMoveStrategy,
}

impl EpsilonGreedyMoveStrategy {
    fn new(player_id: PlayerId) -> Self {
        Self {
            q_table_strat: QTableMoveStrategy {
                player_id,
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

impl MoveStrategy for EpsilonGreedyMoveStrategy {
    fn get_move(&mut self, board: &impl Board) -> BoardIdx {
        if self.rng.random::<f64>() < self.epsilon {
            self.random_strat.get_move(board)
        } else {
            self.q_table_strat.get_move(board)
        }
    }

    fn reset(&mut self) {}
}

pub fn train(params: Params) -> QTable<State, Action, Reward> {
    let mut game = Game::new(
        BitsetBoard::default(),
        EpsilonGreedyMoveStrategy::new(PlayerId::X),
        EpsilonGreedyMoveStrategy::new(PlayerId::O),
    );

    for i in 0..params.training.num_episodes {
        loop {
            match game.advance() {
                GameState::Ongoing { next_player } => todo!(),
                GameState::Won { winner } => todo!(),
                GameState::Tied => todo!(),
            }
        }
    }

    todo!()
}
