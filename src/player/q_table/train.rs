use std::{
    cell::UnsafeCell,
    ops::{Index, IndexMut},
    ptr::NonNull,
};

use ordered_float::OrderedFloat;
use rand::{
    Rng, SeedableRng,
    rngs::{StdRng, ThreadRng},
};

use crate::{
    board::{Board, BoardIdx, bitset::BitsetBoard},
    game::{Game, GameStateRef, GameStatus},
    player::{
        MoveStrategy, PlayerId,
        q_table::{Action, QTable, QTableMoveStrategy, Reward, State, StateAction},
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
}

impl EpsilonGreedyMoveStrategy {
    fn new(params: &ExploreParams, rng_seed: Option<u64>) -> Self {
        Self {
            rng: rng_seed
                .map(StdRng::seed_from_u64)
                .unwrap_or_else(StdRng::from_os_rng),
            epsilon: params.get_epsilon(0),
            q_table_strat: QTableMoveStrategy {
                ..Default::default()
            },
            random_strat: RandomMoveStrategy::default(),
        }
    }
}

impl MoveStrategy for EpsilonGreedyMoveStrategy {
    fn get_move<B: Board>(&mut self, game_state: GameStateRef<'_, B>) -> BoardIdx {
        if self.rng.random::<f64>() < self.epsilon {
            self.random_strat.get_move(game_state)
        } else {
            self.q_table_strat.get_move(game_state)
        }
    }
}

impl<'a> MoveStrategy for &'a UnsafeCell<EpsilonGreedyMoveStrategy> {
    fn get_move<B: Board>(&mut self, game_state: GameStateRef<'_, B>) -> BoardIdx {
        unsafe { (&mut *self.get()).get_move(game_state) }
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
        state_action: StateAction,
        reward: Reward,
        next_state: Option<State>,
        q_table: &mut QTable,
    ) {
        let max_next_q_value = match next_state {
            Some(next_state) => {
                q_table
                    .action_max(next_state, Action::iter_available(&next_state.1))
                    .expect("no action available")
                    .1
                    .0
            }
            None => 0.0,
        };
        let cur_q_value = *q_table[state_action];
        q_table[state_action] +=
            self.learning_rate * (*reward + self.discount_factor * max_next_q_value - cur_q_value);
    }
}

pub fn train(params: Params) -> QTable {
    #[derive(Default)]
    struct StateActions {
        x_sa: Option<StateAction>,
        o_sa: Option<StateAction>,
    }

    impl Index<PlayerId> for StateActions {
        type Output = Option<StateAction>;

        fn index(&self, index: PlayerId) -> &Self::Output {
            match index {
                PlayerId::X => &self.x_sa,
                PlayerId::O => &self.o_sa,
            }
        }
    }

    impl IndexMut<PlayerId> for StateActions {
        fn index_mut(&mut self, index: PlayerId) -> &mut Self::Output {
            match index {
                PlayerId::X => &mut self.x_sa,
                PlayerId::O => &mut self.o_sa,
            }
        }
    }

    let move_strat = UnsafeCell::new(EpsilonGreedyMoveStrategy::new(
        &params.explore,
        params.rng_seed,
    ));
    let update_fn = UpdateFunction::new(params.update);
    let mut game = Game::new(BitsetBoard::default(), &move_strat, &move_strat);

    for i in 0..params.training.num_episodes {
        if i % 1000 == 0 {
            println!("Training episode {i}...");
        }

        let mut prev_sas = StateActions::default();

        let update_q_table = |player: PlayerId,
                              state_action: StateAction,
                              reward: f64,
                              next_board: Option<BitsetBoard>| {
            unsafe {
                let q_table = &mut (&mut *move_strat.get()).q_table_strat.q_table;
                update_fn.apply_update(
                    state_action,
                    reward.into(),
                    next_board.map(|board| State(player, board)),
                    q_table,
                );
            }
        };

        while !game.status.is_finished() {
            let cur_player = game.cur_player;
            let prev_player = game.turns.last().copied().map(|(player, _)| player);

            // Extract current player state
            let cur_state = State(cur_player, game.board);

            // Advance game
            game.advance();

            // Extract current player action
            let cur_action = Action(game.turns.last().copied().unwrap().1);

            // Save current player state-action
            prev_sas[cur_player] = Some(StateAction::new(cur_state, cur_action));

            // Compute reward and update q-table for previous player
            if let Some(prev_player) = prev_player {
                let reward = match game.status {
                    GameStatus::Ongoing | GameStatus::Tied => 0.0,
                    GameStatus::Won => -1.0, // Previous player lost
                };
                update_q_table(
                    prev_player,
                    prev_sas[prev_player].unwrap(),
                    reward,
                    Some(game.board),
                );
            }
        }

        // Apply terminal update for winning player
        update_q_table(
            game.cur_player,
            prev_sas[game.cur_player].unwrap(),
            1.0,
            None,
        );
    }

    move_strat.into_inner().q_table_strat.q_table
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
