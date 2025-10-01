use anyhow::Result;
use tic_tac_toe::player::q_table::{
    QTable,
    train::{ExploreParams, LearningParams, Params, TrainingParams, UpdateParams, train},
};

fn main() -> Result<()> {
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
            num_episodes: 200_000,
            max_steps_per_episode: 9,
        },
        explore: ExploreParams::EpsilonDecay {
            e_start: 1.0,
            e_min: 0.05,
            decay_frac: 0.8,
        },
        rng_seed: None,
    };

    println!("Starting Q-learning training for tic-tac-toe...");
    println!("Episodes: {}", params.training.num_episodes);

    let (q_table, eval_stats) = train(params);

    println!("\nTraining complete!");
    println!("Final performance:");
    println!(
        "  vs Random: {:.1}% win rate",
        eval_stats.random.win_rate() * 100.0
    );
    println!(
        "  vs Suboptimal: {:.1}% win rate",
        eval_stats.suboptimal.win_rate() * 100.0
    );

    q_table.write_to_file(QTable::DEFAULT_FILE_PATH)?;
    println!("\nQ-table saved to {}", QTable::DEFAULT_FILE_PATH);

    Ok(())
}
