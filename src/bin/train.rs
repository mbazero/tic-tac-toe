use anyhow::Result;
use tic_tac_toe::player::q_table::{
    QTable,
    train::{ExploreParams, Params, TrainingParams, UpdateParams, train},
};

fn main() -> Result<()> {
    let params = Params {
        update: UpdateParams {
            discount_factor: 1.0,
            learning_rate: 0.1,
        },
        training: TrainingParams {
            num_episodes: 100_000,
            max_steps_per_episode: 9,
        },
        explore: ExploreParams::EpsilonDecay {
            epsilon_start: 1.0,
            epsilon_end: 0.1,
            decay_steps: 80_000,
        },
        rng_seed: Some(42),
    };

    let (q_table, _) = train(params);
    q_table.write_to_file(QTable::DEFAULT_FILE_PATH)?;
    Ok(())
}
