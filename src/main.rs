use anyhow::Result;
use clap::Parser;
use strum::{Display, EnumString};

use crate::{
    board::array::ArrayBoard,
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
    fn into_strategy(self, player: PlayerId) -> MoveStrategyEnum {
        match self {
            MoveStrategyArg::Human => HumanMoveStrategy::new(player).into(),
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
        ArrayBoard::default(),
        args.x_strat.into_strategy(PlayerId::X),
        args.o_strat.into_strategy(PlayerId::O),
    );

    println!("\n{game}\n");
    while !game.advance().is_finished() {
        println!("{game}\n");
    }
    println!("\n{game}");

    Ok(())
}
