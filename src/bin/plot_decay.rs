use anyhow::Result;
use clap::Parser;
use tic_tac_toe::player::q_table::train::{
    EpsilonProvider, ExploreParams, LearningParams, LearningRateProvider,
};

#[derive(Parser)]
#[command(about = "Plot epsilon and learning rate decay curves")]
struct Args {
    #[arg(short = 'n', long, default_value_t = 10000)]
    num_episodes: u64,

    #[arg(long, default_value_t = 1.0)]
    epsilon_start: f64,

    #[arg(long, default_value_t = 0.05)]
    epsilon_min: f64,

    #[arg(long, default_value_t = 0.8)]
    epsilon_decay_frac: f64,

    #[arg(long, default_value_t = 0.1)]
    alpha_start: f64,

    #[arg(long, default_value_t = 0.01)]
    alpha_min: f64,

    #[arg(long, default_value_t = 0.9)]
    alpha_decay_frac: f64,

    #[arg(long, default_value_t = 0.85)]
    power: f64,
}

fn main() -> Result<()> {
    let Args {
        num_episodes,
        epsilon_start,
        epsilon_min,
        epsilon_decay_frac,
        alpha_start,
        alpha_min,
        alpha_decay_frac,
        power,
    } = Args::parse();

    // Create epsilon provider
    let epsilon_provider = EpsilonProvider::new(
        num_episodes,
        ExploreParams::EpsilonDecay {
            e_start: epsilon_start,
            e_min: epsilon_min,
            decay_frac: epsilon_decay_frac,
        },
    );

    // Create learning rate provider
    let lr_provider = LearningRateProvider::new(
        num_episodes,
        LearningParams::PowerDecay {
            alpha_start,
            alpha_min,
            decay_frac: alpha_decay_frac,
            power,
        },
    );

    // Create combined plot
    plot_combined_decay(num_episodes, &epsilon_provider, &lr_provider)?;

    Ok(())
}

fn plot_combined_decay(
    n: u64,
    epsilon_provider: &EpsilonProvider,
    lr_provider: &LearningRateProvider,
) -> Result<()> {
    use plotters::{
        chart::ChartBuilder,
        prelude::{BitMapBackend, IntoDrawingArea},
        series::LineSeries,
        style::{BLACK, BLUE, Color, IntoFont, RED, TextStyle, WHITE},
    };

    let path = tempfile::Builder::new()
        .prefix("decay_curves_")
        .suffix(".png")
        .tempfile()?
        .into_temp_path()
        .to_path_buf();

    let root = BitMapBackend::new(&path, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let epsilon_values: Vec<(u64, f64)> =
        (0..=n).map(|t| (t, epsilon_provider.epsilon(t))).collect();

    let lr_values: Vec<(u64, f64)> = (0..=n).map(|t| (t, lr_provider.learning_rate(t))).collect();

    // Calculate ranges for each axis (min is always 0)
    let epsilon_min = 0.0;
    let epsilon_max = epsilon_values
        .iter()
        .map(|(_, v)| *v)
        .fold(f64::NEG_INFINITY, f64::max);

    let lr_min = 0.0;
    let lr_max = lr_values
        .iter()
        .map(|(_, v)| *v)
        .fold(f64::NEG_INFINITY, f64::max);

    // Create chart with primary y-axis for epsilon
    let mut chart = ChartBuilder::on(&root)
        .caption(
            "Epsilon and Learning Rate Decay",
            ("sans-serif", 30).into_font(),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .right_y_label_area_size(60)
        .build_cartesian_2d(0u64..n, epsilon_min..epsilon_max)?
        .set_secondary_coord(0u64..n, lr_min..lr_max);

    // Configure primary mesh (left axis for epsilon)
    chart
        .configure_mesh()
        .x_desc("Episode")
        .y_desc("Epsilon (Exploration Rate)")
        .y_label_style(TextStyle::from(("sans-serif", 12)).color(&BLUE))
        .draw()?;

    // Configure secondary mesh (right axis for learning rate)
    chart
        .configure_secondary_axes()
        .y_desc("Learning Rate (Alpha)")
        .label_style(TextStyle::from(("sans-serif", 12)).color(&RED))
        .draw()?;

    // Plot epsilon decay in blue on primary axis
    chart
        .draw_series(LineSeries::new(epsilon_values, &BLUE))?
        .label("Epsilon (left axis)")
        .legend(|(x, y)| plotters::element::PathElement::new(vec![(x, y), (x + 10, y)], &BLUE));

    // Plot learning rate decay in red on secondary axis
    chart
        .draw_secondary_series(LineSeries::new(lr_values, &RED))?
        .label("Learning Rate (right axis)")
        .legend(|(x, y)| plotters::element::PathElement::new(vec![(x, y), (x + 10, y)], &RED));

    // Configure and draw legend
    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;

    println!("Saved plot to {path:?}");

    #[cfg(target_os = "macos")]
    std::process::Command::new("open").arg(&path).spawn()?;

    Ok(())
}
