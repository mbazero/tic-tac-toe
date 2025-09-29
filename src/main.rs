use anyhow::Result;
use clap::Parser;
use strum::{Display, EnumString};

use crate::{
    board::bitset::BitsetBoard,
    game::Game,
    player::{
        MoveStrategy, MoveStrategyEnum, PlayerId, human::HumanMoveStrategy,
        random::RandomMoveStrategy,
    },
};

mod board;
mod game;
mod player;

#[derive(Copy, Clone, Debug, EnumString, Default, Display)]
enum MoveStrategyArg {
    #[default]
    #[strum(serialize = "human", serialize = "h")]
    Human,
    #[strum(serialize = "random", serialize = "r")]
    Random,
}

impl MoveStrategyArg {
    fn into_strat(self) -> MoveStrategyEnum {
        match self {
            MoveStrategyArg::Human => HumanMoveStrategy::default().into(),
            MoveStrategyArg::Random => RandomMoveStrategy::default().into(),
        }
    }
}

#[derive(Parser, Debug)]
struct CliArgs {
    #[arg(short = 'x', default_value_t = MoveStrategyArg::Human)]
    x_strat: MoveStrategyArg,
    #[arg(short = 'o', default_value_t = MoveStrategyArg::Random)]
    o_strat: MoveStrategyArg,
}

fn main() -> Result<()> {
    let args = CliArgs::parse();

    println!("Player X: {}", args.x_strat);
    println!("Player O: {}", args.o_strat);

    let mut game = Game::new(
        BitsetBoard::default(),
        args.x_strat.into_strat(),
        args.o_strat.into_strat(),
    );

    println!("\n{game}\n");
    while !game.advance().is_finished() {
        println!("{game}\n");
    }
    println!("\n{game}");

    Ok(())
}
