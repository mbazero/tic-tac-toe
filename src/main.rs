use anyhow::Result;
use clap::Parser;
use strum::{Display, EnumString};

use tic_tac_toe::{
    board::bitset::BitsetBoard,
    game::Game,
    player::{
        MoveStrategyEnum,
        human::HumanMoveStrategy,
        q_table::{QTable, QTableMoveStrategy},
        random::RandomMoveStrategy,
    },
};

#[derive(Copy, Clone, Debug, EnumString, Default, Display)]
enum MoveStrategyArg {
    #[default]
    #[strum(serialize = "human", serialize = "h")]
    Human,
    #[strum(serialize = "random", serialize = "r")]
    Random,
    #[strum(serialize = "qtable", serialize = "q")]
    QTable,
}

impl MoveStrategyArg {
    fn into_strat(self) -> Result<MoveStrategyEnum> {
        match self {
            MoveStrategyArg::Human => Ok(HumanMoveStrategy::default().into()),
            MoveStrategyArg::Random => Ok(RandomMoveStrategy::default().into()),
            MoveStrategyArg::QTable => {
                let q_table = QTable::read_from_file(QTable::DEFAULT_FILE_PATH)?;
                Ok(QTableMoveStrategy::new(q_table).into())
            }
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
        args.x_strat.into_strat()?,
        args.o_strat.into_strat()?,
    );

    println!("\n{game}\n");
    while !game.advance().is_finished() {
        println!("{game}\n");
    }
    println!("\n{game}");

    Ok(())
}
