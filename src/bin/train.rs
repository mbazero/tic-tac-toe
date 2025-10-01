use anyhow::Result;
use tic_tac_toe::player::q_table::{
    QTable,
    train::{ExploreParams, LearningParams, Params, TrainingParams, UpdateParams, train},
};

fn main() -> Result<()> {
    let params = Params {
        update: UpdateParams {
            // Discount factor: 1.0 is optimal for episodic games like tic-tac-toe
            // where future rewards are equally important as immediate rewards
            discount_factor: 1.0,
            // Learning rate: Power decay provides better convergence than fixed rate
            learning_rate: LearningParams::PowerDecay {
                // Start with moderate learning rate for stable initial learning
                alpha_start: 0.15,
                // Minimum ensures continued refinement in late training
                alpha_min: 0.005,
                // Decay over 90% of episodes for thorough exploration before settling
                decay_frac: 0.9,
                // Power of 0.85 provides good balance between exploration and convergence
                power: 0.85,
            },
        },
        training: TrainingParams {
            // 200k episodes ensures thorough exploration of state space
            // Tic-tac-toe has ~5,478 unique states, so this allows ~36 visits per state
            num_episodes: 200_000,
            // Max 9 moves in tic-tac-toe
            max_steps_per_episode: 9,
        },
        explore: ExploreParams::EpsilonDecay {
            // Start with full exploration to discover all states
            e_start: 1.0,
            // Lower minimum epsilon (0.01) for more exploitation in late training
            // This produces a stronger final policy
            e_min: 0.01,
            // Decay over 70% of episodes, allowing more exploitation time
            // than learning rate decay to refine the policy
            decay_frac: 0.7,
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
