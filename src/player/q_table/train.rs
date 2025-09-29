use std::cell::UnsafeCell;

use enum_map::EnumMap;
use rand::{Rng, SeedableRng, rngs::StdRng};

use crate::{
    board::{Board, BoardIdx, bitset::BitsetBoard},
    game::{Game, GameStateRef, GameStatus},
    player::{
        MoveStrategy, PlayerId,
        q_table::{Action, QTable, QTableMoveStrategy, Reward, State, StateAction},
        random::RandomMoveStrategy,
    },
};

pub struct Params {
    pub update: UpdateParams,
    pub training: TrainingParams,
    pub explore: ExploreParams,
    pub rng_seed: Option<u64>,
}

pub struct UpdateParams {
    pub discount_factor: f64, // gamma
    pub learning_rate: f64,   // alpha
}

pub struct TrainingParams {
    pub num_episodes: u64,
    pub max_steps_per_episode: u64,
}

pub enum ExploreParams {
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
    pub fn get_epsilon(&self, episode: u64) -> f64 {
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

impl MoveStrategy for &UnsafeCell<EpsilonGreedyMoveStrategy> {
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
                match q_table.action_max(next_state, Action::iter_available(&next_state.1)) {
                    Some((_, q_value)) => q_value.0,
                    None => panic!(
                        "failed to find next action\nplayer: {:?}\nprev_board:\n{}\naction: {}\nnext_board:\n{}",
                        state_action.player_id(),
                        state_action.board(),
                        state_action.action().0,
                        next_state.1,
                    ),
                }
            }
            None => 0.0,
        };
        let cur_q_value = *q_table[state_action];
        q_table[state_action] +=
            self.learning_rate * (*reward + self.discount_factor * max_next_q_value - cur_q_value);
    }
}

pub fn train(params: Params) -> (QTable, EvalStats) {
    // TODO: Only wrap QTable in unsafe cell
    let move_strat = UnsafeCell::new(EpsilonGreedyMoveStrategy::new(
        &params.explore,
        params.rng_seed,
    ));
    let update_fn = UpdateFunction::new(params.update);
    let mut eval_stats = EvalStats::default();

    for i in 0..params.training.num_episodes {
        let mut game = Game::new(BitsetBoard::default(), &move_strat, &move_strat);
        let mut prev_sas = EnumMap::default();

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
                let (reward, next_board) = match game.status {
                    GameStatus::Ongoing => (0.0, Some(game.board)),
                    GameStatus::Tied => (0.0, None),
                    GameStatus::Won => (-1.0, None),
                };

                update_q_table(
                    prev_player,
                    prev_sas[prev_player].unwrap(),
                    reward,
                    next_board,
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

        if (i + 1) % 1000 == 0 {
            eval_stats = eval(500, &move_strat);
            println!("Episode {} win rate: {}", i + 1, eval_stats.win_rate());
        }
    }

    (move_strat.into_inner().q_table_strat.q_table, eval_stats)
}

#[derive(Default, Clone, Eq, PartialEq)]
pub struct EvalStats {
    win_count: usize,
    tie_count: usize,
    loss_count: usize,
}

impl EvalStats {
    pub fn total_count(&self) -> usize {
        self.win_count + self.tie_count + self.loss_count
    }

    pub fn win_rate(&self) -> f64 {
        self.win_count as f64 / self.total_count() as f64
    }

    pub fn tie_rate(&self) -> f64 {
        self.tie_count as f64 / self.total_count() as f64
    }

    pub fn loss_rate(&self) -> f64 {
        self.loss_count as f64 / self.total_count() as f64
    }
}

fn eval(num_games: usize, move_strat: &UnsafeCell<EpsilonGreedyMoveStrategy>) -> EvalStats {
    let old_epsilon = unsafe {
        let move_strat = &mut *move_strat.get();
        std::mem::replace(&mut move_strat.epsilon, 0.0)
    };

    let stats = (0..num_games).fold(EvalStats::default(), |mut stats, _| {
        let mut game = Game::new(
            BitsetBoard::default(),
            move_strat,
            RandomMoveStrategy::default(),
        );

        while !game.status.is_finished() {
            game.advance();
        }

        match game.status {
            GameStatus::Ongoing => unreachable!(),
            GameStatus::Won => match game.cur_player {
                PlayerId::X => stats.win_count += 1,
                PlayerId::O => stats.loss_count += 1,
            },
            GameStatus::Tied => stats.tie_count += 1,
        }

        stats
    });

    unsafe {
        (&mut *move_strat.get()).epsilon = old_epsilon;
    }

    stats
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

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
        let _ = train(params);
        Ok(())
    }
}
