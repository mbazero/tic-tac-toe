use std::cell::UnsafeCell;

use rand::{Rng, SeedableRng, rngs::SmallRng};

use crate::{
    board::{Board, BoardIdx, bitset::BitsetBoard},
    game::{Game, GameStateRef, GameStatus},
    player::{
        MoveStrategy, PlayerId,
        q_table::{Action, QTable, QTableMoveStrategy, Reward, State, StateAction},
        random::RandomMoveStrategy,
        suboptimal::SuboptimalMoveStrategy,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub struct Params {
    pub update: UpdateParams,
    pub training: TrainingParams,
    pub explore: ExploreParams,
    pub rng_seed: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateParams {
    pub discount_factor: f64,          // gamma
    pub learning_rate: LearningParams, // alpha
}

#[derive(Debug, Clone, PartialEq)]
pub enum LearningParams {
    Fixed(f64),
    PowerDecay {
        alpha_start: f64,
        alpha_min: f64,
        decay_frac: f64,
        power: f64,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrainingParams {
    pub num_episodes: u64,
    pub max_steps_per_episode: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExploreParams {
    FixedEpsilon(f64),
    EpsilonDecay {
        e_start: f64,
        e_min: f64,
        decay_frac: f64,
    },
}

pub enum EpsilonProvider {
    Fixed(f64),
    ExponentialDecay {
        e_start: f64,
        e_min: f64,
        k: f64,
        t_floor: u64,
    },
}

impl EpsilonProvider {
    pub fn new(n: u64, params: ExploreParams) -> Self {
        match params {
            ExploreParams::FixedEpsilon(e) => Self::Fixed(e),
            ExploreParams::EpsilonDecay {
                e_start,
                e_min,
                decay_frac,
            } => {
                assert!(e_start > e_min && e_min > 0.0 && (0.0..=1.0).contains(&decay_frac));
                let t_floor = (decay_frac * n as f64).ceil().max(1.0);
                let k = -(e_min / e_start).ln() / t_floor;
                Self::ExponentialDecay {
                    e_start,
                    e_min,
                    k,
                    t_floor: t_floor as u64,
                }
            }
        }
    }

    pub fn epsilon(&self, episode: u64) -> f64 {
        match *self {
            EpsilonProvider::Fixed(e) => e,
            EpsilonProvider::ExponentialDecay {
                e_start,
                e_min,
                k,
                t_floor,
            } => {
                if episode >= t_floor {
                    e_min
                } else {
                    e_start * (-k * episode as f64).exp()
                }
            }
        }
    }

    #[cfg(feature = "plotting")]
    pub fn plot(&self, n: u64, path: &impl AsRef<std::path::Path>) -> anyhow::Result<()> {
        use plotters::{
            chart::ChartBuilder,
            prelude::{BitMapBackend, IntoDrawingArea},
            series::LineSeries,
            style::{BLUE, WHITE},
        };

        let root = BitMapBackend::new(path, (1024, 768)).into_drawing_area();
        root.fill(&WHITE)?;

        let eps: Vec<(u64, f64)> = (0..=n).map(|t| (t, self.epsilon(t))).collect();

        let y_min = 0.0;
        let y_max = eps
            .iter()
            .map(|(_, e)| *e)
            .fold(f64::NEG_INFINITY, f64::max);

        let mut chart = ChartBuilder::on(&root)
            .caption("Epsilon Decay", ("sans-serif", 30))
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(0u64..n, y_min..y_max)?;

        chart
            .configure_mesh()
            .x_desc("Episode")
            .y_desc("Epsilon")
            .draw()?;
        chart.draw_series(LineSeries::new(eps, &BLUE))?;
        root.present()?;
        Ok(())
    }
}

pub enum LearningRateProvider {
    Fixed(f64),
    PowerDecay {
        alpha_start: f64,
        alpha_min: f64,
        k: f64,
        t_floor: u64,
        power: f64,
    },
}

impl LearningRateProvider {
    pub fn new(n: u64, params: LearningParams) -> Self {
        match params {
            LearningParams::Fixed(alpha) => Self::Fixed(alpha),
            LearningParams::PowerDecay {
                alpha_start,
                alpha_min,
                decay_frac,
                power,
            } => {
                assert!(
                    alpha_start > alpha_min
                        && alpha_min > 0.0
                        && (0.0..=1.0).contains(&decay_frac)
                        && power > 0.0
                );
                let t_floor = (decay_frac * n as f64).ceil().max(1.0);
                // Compute k such that at t_floor: alpha_start / (1 + k * t_floor)^power ≈ alpha_min
                // Solving: (1 + k * t_floor)^power = alpha_start / alpha_min
                // k = ((alpha_start / alpha_min)^(1/power) - 1) / t_floor
                let k = ((alpha_start / alpha_min).powf(1.0 / power) - 1.0) / t_floor;
                Self::PowerDecay {
                    alpha_start,
                    alpha_min,
                    k,
                    t_floor: t_floor as u64,
                    power,
                }
            }
        }
    }

    pub fn learning_rate(&self, episode: u64) -> f64 {
        match *self {
            LearningRateProvider::Fixed(alpha) => alpha,
            LearningRateProvider::PowerDecay {
                alpha_start,
                alpha_min,
                k,
                t_floor,
                power,
            } => {
                if episode >= t_floor {
                    alpha_min
                } else {
                    let alpha = alpha_start / (1.0 + k * episode as f64).powf(power);
                    alpha.max(alpha_min)
                }
            }
        }
    }

    #[cfg(feature = "plotting")]
    pub fn plot(&self, n: u64, path: &impl AsRef<std::path::Path>) -> anyhow::Result<()> {
        use plotters::{
            chart::ChartBuilder,
            prelude::{BitMapBackend, IntoDrawingArea},
            series::LineSeries,
            style::{RED, WHITE},
        };

        let root = BitMapBackend::new(path, (1024, 768)).into_drawing_area();
        root.fill(&WHITE)?;

        let alphas: Vec<(u64, f64)> = (0..=n).map(|t| (t, self.learning_rate(t))).collect();

        let y_min = 0.0;
        let y_max = alphas
            .iter()
            .map(|(_, a)| *a)
            .fold(f64::NEG_INFINITY, f64::max);

        let mut chart = ChartBuilder::on(&root)
            .caption("Learning Rate Decay", ("sans-serif", 30))
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(0u64..n, y_min..y_max)?;

        chart
            .configure_mesh()
            .x_desc("Episode")
            .y_desc("Learning Rate")
            .draw()?;
        chart.draw_series(LineSeries::new(alphas, &RED))?;
        root.present()?;
        Ok(())
    }
}

struct EpsilonGreedyMoveStrategy {
    rng: SmallRng,
    epsilon: f64,
    epsilon_frozen: Option<f64>,
    epsilon_provider: EpsilonProvider,
    q_table_strat: QTableMoveStrategy,
    random_strat: RandomMoveStrategy,
}

impl EpsilonGreedyMoveStrategy {
    fn new(epsilon_provider: EpsilonProvider, rng: SmallRng) -> Self {
        Self {
            rng: rng.clone(),
            epsilon: epsilon_provider.epsilon(0),
            epsilon_frozen: None,
            epsilon_provider,
            q_table_strat: QTableMoveStrategy {
                q_table: Default::default(),
            },
            random_strat: RandomMoveStrategy::new(rng),
        }
    }

    fn set_episode(&mut self, episode: u64) {
        self.epsilon = self.epsilon_provider.epsilon(episode);
    }

    fn freeze(&mut self) {
        assert!(
            self.epsilon_frozen.is_none(),
            "exploration is already frozen"
        );
        self.epsilon_frozen = Some(self.epsilon);
        self.epsilon = 0.0;
    }

    fn unfreeze(&mut self) {
        assert!(
            self.epsilon == 0.0 && self.epsilon_frozen.is_some(),
            "exploration is not frozen"
        );
        self.epsilon = self.epsilon_frozen.take().unwrap();
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
    lr_provider: LearningRateProvider,
    learning_rate: f64,
}

impl UpdateFunction {
    fn new(params: UpdateParams, n_episodes: u64) -> Self {
        let lr_provider = LearningRateProvider::new(n_episodes, params.learning_rate);
        let current_learning_rate = lr_provider.learning_rate(0);
        Self {
            discount_factor: params.discount_factor,
            lr_provider,
            learning_rate: current_learning_rate,
        }
    }

    fn set_episode(&mut self, episode: u64) {
        self.learning_rate = self.lr_provider.learning_rate(episode);
    }

    fn apply_update(
        &self,
        state_action: StateAction,
        reward: Reward,
        next_state: Option<State>,
        q_table: &mut QTable,
    ) {
        let max_next_q_value = match next_state {
            Some(next_state) => q_table
                .max_q(next_state)
                .expect("max q should exist")
                .into_inner(),
            None => 0.0,
        };
        let cur_q_value = *q_table[state_action];
        q_table[state_action] +=
            self.learning_rate * (*reward + self.discount_factor * -max_next_q_value - cur_q_value);
    }
}

pub fn train(params: Params) -> (QTable, CombinedEvalStats) {
    let Params {
        update,
        training,
        explore,
        rng_seed,
    } = params;

    let rng = rng_seed
        .map(SmallRng::seed_from_u64)
        .unwrap_or_else(SmallRng::from_os_rng);

    // TODO: Only wrap QTable in unsafe cell
    let mut move_strat = UnsafeCell::new(EpsilonGreedyMoveStrategy::new(
        EpsilonProvider::new(training.num_episodes, explore),
        rng.clone(),
    ));
    let mut update_fn = UpdateFunction::new(update, training.num_episodes);
    let mut eval_stats = CombinedEvalStats::default();

    for i in 0..training.num_episodes {
        move_strat.get_mut().set_episode(i);
        update_fn.set_episode(i);

        let mut game = Game::new(BitsetBoard::default(), &move_strat, &move_strat);

        let update_q_table = |state_action: StateAction, reward: f64, next_state: Option<State>| unsafe {
            let q_table = &mut (&mut *move_strat.get()).q_table_strat.q_table;
            update_fn.apply_update(state_action, reward.into(), next_state, q_table);
        };

        while !game.status.is_finished() {
            let cur_player = game.cur_player;

            // Extract current player state
            let cur_state = State(cur_player, game.board);

            // Advance game
            game.advance();

            // Extract current player action
            let cur_action = Action(game.turns.last().copied().unwrap().1);
            let state_action = StateAction::new(cur_state, cur_action);

            let (reward, next_state) = match game.status {
                GameStatus::Won => (1.0, None),
                GameStatus::Tied => (0.0, None),
                GameStatus::Ongoing => (0.0, Some(State(cur_player.other(), game.board))),
            };

            update_q_table(state_action, reward, next_state);
        }

        if (i + 1) % 1000 == 0 {
            eval_stats.random = eval_as_x(1000, &mut move_strat, random_opponent);
            eval_stats.suboptimal = eval_as_x(1000, &mut move_strat, suboptimal_opponent);
            println!(
                "Episode {:6.0} win rate: {:.2} | {:.2}",
                i + 1,
                eval_stats.random.win_rate(),
                eval_stats.suboptimal.win_rate(),
            );
        }
    }

    (move_strat.into_inner().q_table_strat.q_table, eval_stats)
}

#[derive(Default, Clone, Eq, PartialEq)]
pub struct CombinedEvalStats {
    pub random: EvalStats,
    pub suboptimal: EvalStats,
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

fn eval_as_x<M: MoveStrategy>(
    num_games: usize,
    e_greedy_strat: &mut UnsafeCell<EpsilonGreedyMoveStrategy>,
    opponent_supplier: fn(usize) -> M,
) -> EvalStats {
    e_greedy_strat.get_mut().freeze();

    let stats = (0..num_games).fold(EvalStats::default(), |mut stats, game_idx| {
        let mut game = Game::new(
            BitsetBoard::default(),
            &*e_greedy_strat,
            opponent_supplier(game_idx),
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

    e_greedy_strat.get_mut().unfreeze();

    stats
}

fn random_opponent(game_idx: usize) -> RandomMoveStrategy {
    const EVAL_RANDOM_SEED: u64 = 0xC0FF_EE00_42AA_F00D;
    let rng = SmallRng::seed_from_u64(EVAL_RANDOM_SEED.wrapping_add(game_idx as u64));
    RandomMoveStrategy::new(rng)
}

fn suboptimal_opponent(game_idx: usize) -> SuboptimalMoveStrategy {
    SuboptimalMoveStrategy::new(random_opponent(game_idx))
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::{
        board::{Board, bitset::BitsetBoard},
        game::GameStateRef,
        player::{
            MoveStrategy, PlayerId,
            q_table::{
                QTableMoveStrategy,
                train::{
                    CombinedEvalStats, EpsilonProvider, ExploreParams, LearningRateProvider,
                    Params, train,
                },
            },
        },
    };

    use super::{LearningParams, TrainingParams, UpdateParams};

    #[test]
    fn test_train() -> Result<()> {
        let params = Params {
            update: UpdateParams {
                discount_factor: 1.0,
                learning_rate: LearningParams::PowerDecay {
                    alpha_start: 0.1,
                    alpha_min: 0.01,
                    decay_frac: 0.8,
                    power: 0.85,
                },
            },
            training: TrainingParams {
                num_episodes: 50_000,
                max_steps_per_episode: 9,
            },
            explore: ExploreParams::EpsilonDecay {
                e_start: 1.0,
                e_min: 0.05,
                decay_frac: 0.8,
            },
            rng_seed: None,
        };

        let (q_table, CombinedEvalStats { suboptimal, .. }) = train(params);

        assert_eq!(
            1.0,
            suboptimal.win_rate(),
            "agent should have an 100% win rate against suboptimal strategy when playing as X",
        );

        let mut q_table_strat = QTableMoveStrategy::new(q_table);
        let first_move = q_table_strat.get_move(GameStateRef {
            cur_player: PlayerId::X,
            board: &BitsetBoard::default(),
            turns: &[],
        });
        assert_eq!(4, first_move, "first move should be center");

        let response_board = BitsetBoard::with_positions([4], [5])?;
        let second_move = q_table_strat.get_move(GameStateRef {
            cur_player: PlayerId::X,
            board: &response_board,
            turns: &[(PlayerId::X, 4), (PlayerId::O, 5)],
        });
        assert!(
            matches!(second_move, 0 | 2 | 6 | 8),
            "after taking center against a side response, agent should pick a corner, got {second_move}",
        );
        Ok(())
    }

    #[test]
    fn test_epsilon_provider() {
        let n = 1_000;
        let e_start = 1.0;
        let e_min = 0.1;
        let decay_frac = 0.5;

        let exp = EpsilonProvider::new(
            n,
            ExploreParams::EpsilonDecay {
                e_start,
                e_min,
                decay_frac,
            },
        );

        // At t = 0 ~ e_start
        assert_eq!(exp.epsilon(0), e_start);

        // Monotone decreasing before floor
        let e100 = exp.epsilon(100);
        let e200 = exp.epsilon(200);
        let e400 = exp.epsilon(400);
        assert!(
            e100 > e200 && e200 > e400,
            "not strictly decreasing: {e100} {e200} {e400}"
        );

        // Just before t_floor still above e_min
        let e499 = exp.epsilon(499);
        assert!(
            e499 > e_min,
            "should be above e_min before floor, got {e499}"
        );

        // At and after t_floor -> clamped to e_min (exact)
        assert_eq!(exp.epsilon(500), e_min);
        assert_eq!(exp.epsilon(600), e_min);
    }

    #[test]
    fn test_learning_rate_provider() {
        let n = 10_000;
        let alpha_start = 0.1;
        let alpha_min = 0.01;
        let decay_frac = 0.8;
        let power = 0.85;

        let lr_provider = LearningRateProvider::new(
            n,
            LearningParams::PowerDecay {
                alpha_start,
                alpha_min,
                decay_frac,
                power,
            },
        );

        // At t = 0, should be alpha_start
        assert_eq!(lr_provider.learning_rate(0), alpha_start);

        // Should decrease monotonically before floor
        let lr1000 = lr_provider.learning_rate(1000);
        let lr3000 = lr_provider.learning_rate(3000);
        let lr5000 = lr_provider.learning_rate(5000);
        let lr7000 = lr_provider.learning_rate(7000);

        assert!(
            lr1000 > lr3000 && lr3000 > lr5000 && lr5000 > lr7000,
            "not strictly decreasing: {lr1000} {lr3000} {lr5000} {lr7000}"
        );

        // Should be above minimum before floor
        let lr7999 = lr_provider.learning_rate(7999);
        assert!(
            lr7999 > alpha_min,
            "should be above alpha_min before floor, got {lr7999}"
        );

        // At and after t_floor (8000) should be clamped to alpha_min
        assert_eq!(lr_provider.learning_rate(8000), alpha_min);
        assert_eq!(lr_provider.learning_rate(9000), alpha_min);
        assert_eq!(lr_provider.learning_rate(10000), alpha_min);

        // Test fixed learning rate variant
        let fixed_lr = LearningRateProvider::new(n, LearningParams::Fixed(0.05));
        assert_eq!(fixed_lr.learning_rate(0), 0.05);
        assert_eq!(fixed_lr.learning_rate(5000), 0.05);
        assert_eq!(fixed_lr.learning_rate(10000), 0.05);
    }
}
