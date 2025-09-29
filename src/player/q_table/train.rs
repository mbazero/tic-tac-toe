use std::ptr::NonNull;

use rand::{
    Rng, SeedableRng,
    rngs::{StdRng, ThreadRng},
};

use crate::{
    board::{Board, BoardIdx, bitset::BitsetBoard},
    game::{Game, GameState},
    player::{
        MoveStrategy, PlayerId,
        q_table::{Action, QTable, QTableMoveStrategy, State, StateAction},
        random::RandomMoveStrategy,
    },
};

struct Params {
    update: UpdateParams,
    training: TrainingParams,
    explore: ExploreParams,
    rng_seed: Option<u64>,
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

struct EpsilonGreedyMoveStrategy {
    rng: StdRng,
    epsilon: f64,
    q_table_strat: QTableMoveStrategy,
    random_strat: RandomMoveStrategy,
    // last_move: Option<BoardIdx>,
}

impl EpsilonGreedyMoveStrategy {
    fn new(player_id: PlayerId, params: &ExploreParams, rng_seed: Option<u64>) -> Self {
        Self {
            rng: rng_seed
                .map(StdRng::seed_from_u64)
                .unwrap_or_else(|| StdRng::from_os_rng()),
            epsilon: params.get_epsilon(0),
            q_table_strat: QTableMoveStrategy {
                player_id,
                ..Default::default()
            },
            random_strat: RandomMoveStrategy::default(),
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
}

impl MoveStrategy for NonNull<EpsilonGreedyMoveStrategy> {
    fn get_move(&mut self, board: &impl Board) -> BoardIdx {
        unsafe { self.as_mut().get_move(board) }
    }
}

struct UpdateFunction {
    discount_factor: f64,
    learning_rate: f64,
}

impl UpdateFunction {
    fn new(params: UpdateParams) -> Self {
        Self {
            discount_factor: params.discount_factor,
            learning_rate: params.learning_rate,
        }
    }

    fn apply_update(
        &self,
        reward: f64,
        cur_sa: StateAction,
        next_state: Option<State>,
        q_table: &mut QTable,
    ) {
        let max_next_q_value = match next_state {
            Some(next_state) => {
                q_table
                    .action_max(next_state, Action::iter_available(&next_state.1))
                    .expect("no action available")
                    .1
            }
            None => 0.0,
        };
        q_table[cur_sa] += self.learning_rate
            * (reward + self.discount_factor * max_next_q_value - q_table[cur_sa]);
    }
}

pub fn train(params: Params) -> QTable {
    let mut move_strat =
        EpsilonGreedyMoveStrategy::new(PlayerId::X, &params.explore, params.rng_seed);
    let q_table = &mut move_strat.q_table_strat.q_table;
    let update_fn = UpdateFunction::new(params.update);

    let mut game = Game::new(
        BitsetBoard::default(),
        NonNull::from_ref(&move_strat),
        NonNull::from_ref(&move_strat),
    );

    for i in 0..params.training.num_episodes {
        while !game.state.is_finished() {
            let prev_board = game.board;
            game.advance();
            let next_board = game.board;

            // let reward = match game.state {
            //     GameState::Ongoing | GameState::Tied => 0.0,
            //     GameState::Won => match game.cur_player {
            //         PlayerId::X => (1.0, -1.0),
            //         PlayerId::O => (-1.0, 1.0),
            //     },
            // };

            // update_fn.apply_update(reward, cur_sa, next_state, q_table);
        }
    }

    todo!()
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use std::{fs::File, io::Write};

    use crate::player::q_table::train::{Params, train};

    use super::{ExploreParams, TrainingParams, UpdateParams};

    #[test]
    fn test_train() -> Result<()> {
        let params = Params {
            update: UpdateParams {
                discount_factor: 1.0,
                learning_rate: 0.1,
            },
            training: TrainingParams {
                num_episodes: 10_000,
                max_steps_per_episode: 9,
            },
            explore: ExploreParams::EpsilonDecay {
                epsilon_start: 1.0,
                epsilon_end: 0.1,
                epsilon_decay: 0.001,
            },
            rng_seed: Some(42),
        };

        let q_table = train(params);

        let mut f = File::create("q_table")?;
        let bytes = bytemuck::cast_slice(q_table.0.as_slice());
        f.write_all(bytes)?;

        Ok(())
    }
}
