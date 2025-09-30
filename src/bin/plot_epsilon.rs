use anyhow::Result;
use clap::Parser;
use tic_tac_toe::player::q_table::train::{EpsilonProvider, ExploreParams};

#[derive(Parser)]
struct Args {
    #[arg(short = 'n', long, default_value_t = 1000)]
    num_epsiodes: u64,

    #[arg(short = 'f', long, default_value_t = 0.8)]
    decay_frac: f64,
}

fn main() -> Result<()> {
    let Args {
        num_epsiodes,
        decay_frac,
    } = Args::parse();

    let exp = EpsilonProvider::new(
        num_epsiodes,
        ExploreParams::EpsilonDecay {
            e_start: 1.0,
            e_min: 0.1,
            decay_frac,
        },
    );

    let path = tempfile::Builder::new()
        .prefix("epislon_")
        .suffix(".png")
        .tempfile()?
        .into_temp_path()
        .to_path_buf();

    exp.plot(num_epsiodes, &path)?;

    println!("Saved plot to {path:?}");

    #[cfg(target_os = "macos")]
    std::process::Command::new("open").arg(&path).spawn()?;

    Ok(())
}
