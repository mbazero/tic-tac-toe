use anyhow::{Context, Result};
use clap::Parser;
use rand::{rngs::ThreadRng, seq::IndexedRandom};
use regex::Regex;
use std::{
    fmt::Display,
    io::{self, Write},
    str::FromStr,
    sync::LazyLock,
};
use strum::{Display, EnumString};
use thiserror::Error;

type BoardIdx = u8;

#[derive(Error, Debug)]
pub enum MoveError {
    #[error("cell is out of bounds")]
    CellOutOfBounds,
    #[error("cell is occupied")]
    CellOccupied,
}

pub struct Game {
    state: GameState,
    board: [Option<PlayerId>; 9],
    moves: usize,
}

impl Game {
    pub fn make_move(&mut self, idx: BoardIdx) -> Result<(), MoveError> {
        let GameState::Ongoing {
            next_player: cur_player,
        } = self.state
        else {
            return Ok(());
        };

        if idx >= 9 {
            return Err(MoveError::CellOutOfBounds);
        }

        if self.board[idx as usize].is_some() {
            return Err(MoveError::CellOccupied);
        }

        self.board[idx as usize] = Some(cur_player);
        self.moves += 1;

        let win_lanes = match idx {
            0 => vec![[0, 1, 2], [0, 4, 8], [0, 3, 6]],
            1 => vec![[0, 1, 2], [1, 4, 7]],
            2 => vec![[0, 1, 2], [6, 4, 2], [2, 5, 8]],
            3 => vec![[3, 4, 5], [0, 3, 6]],
            4 => vec![[3, 4, 5], [0, 4, 8], [6, 4, 2], [1, 4, 7]],
            5 => vec![[3, 4, 5], [2, 5, 8]],
            6 => vec![[6, 7, 8], [0, 3, 6], [6, 4, 2]],
            7 => vec![[6, 7, 8], [1, 4, 7]],
            8 => vec![[6, 7, 8], [0, 4, 8], [2, 5, 8]],
            _ => unreachable!(),
        };

        if win_lanes
            .into_iter()
            .any(|lane| lane.into_iter().all(|i| self.board[i] == Some(cur_player)))
        {
            self.state = GameState::Win { winner: cur_player }
        } else if self.moves == 9 {
            self.state = GameState::Tie;
        } else {
            self.state = GameState::Ongoing {
                next_player: match cur_player {
                    PlayerId::X => PlayerId::O,
                    PlayerId::O => PlayerId::X,
                },
            }
        }

        Ok(())
    }
}

impl Default for Game {
    fn default() -> Self {
        Self {
            state: GameState::Ongoing {
                next_player: PlayerId::X,
            },
            board: [None; 9],
            moves: 0,
        }
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cc = |i: usize| match self.board[i] {
            Some(PlayerId::X) => 'X',
            Some(PlayerId::O) => 'O',
            None => ' ',
        };
        writeln!(f, "{}|{}|{}", cc(0), cc(1), cc(2))?;
        writeln!(f, "-----")?;
        writeln!(f, "{}|{}|{}", cc(3), cc(4), cc(5))?;
        writeln!(f, "-----")?;
        writeln!(f, "{}|{}|{}", cc(6), cc(7), cc(8))?;
        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
pub enum GameState {
    Ongoing { next_player: PlayerId },
    Win { winner: PlayerId },
    Tie,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PlayerId {
    X,
    O,
}

impl Display for PlayerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PlayerId::X => 'X',
                PlayerId::O => 'O',
            }
        )
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Coords {
    i: u8,
    j: u8,
}

impl Coords {
    fn to_board_idx(self) -> BoardIdx {
        self.i * 3 + self.j
    }
}

impl FromStr for Coords {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        static RGX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^([0-2]),([0-2])$").unwrap());
        let caps = RGX
            .captures(s.trim())
            .context("malformed coordinate string")?;
        Ok(Self {
            i: caps[1].parse()?,
            j: caps[2].parse()?,
        })
    }
}

#[derive(Debug, Default)]
struct CliReader {
    buffer: String,
}

impl CliReader {
    fn read<T>(&mut self) -> Result<T>
    where
        T: FromStr,
        Result<T, T::Err>: Context<T, T::Err>,
    {
        io::stdout().flush().unwrap();
        self.buffer.clear();
        io::stdin().read_line(&mut self.buffer).unwrap();
        T::from_str(&self.buffer).context("failed to parse input")
    }
}

pub trait MoveStrategy {
    fn get_move(&mut self, board: &[Option<PlayerId>; 9]) -> BoardIdx;
}

#[derive(Default)]
pub struct HumanMoveStrategy {
    cli_reader: CliReader,
}

impl MoveStrategy for HumanMoveStrategy {
    fn get_move(&mut self, _: &[Option<PlayerId>; 9]) -> BoardIdx {
        loop {
            if let Ok(coords) = self.cli_reader.read::<Coords>() {
                return coords.to_board_idx();
            } else {
                print!("Invalid input; try again: ");
            }
        }
    }
}

#[derive(Default)]
pub struct RandomMoveStrategy {
    rng: ThreadRng,
}

impl MoveStrategy for RandomMoveStrategy {
    fn get_move(&mut self, board: &[Option<PlayerId>; 9]) -> BoardIdx {
        let open_squares: Vec<_> = (0u8..)
            .zip(board)
            .filter_map(|(i, x)| x.is_none().then_some(i))
            .collect();
        *open_squares.choose(&mut self.rng).unwrap()
    }
}

#[derive(Copy, Clone, Debug, EnumString, Default, Display)]
pub enum MoveStrategyId {
    #[default]
    #[strum(serialize = "human", serialize = "h")]
    Human,
    #[strum(serialize = "random", serialize = "r")]
    Random,
}

impl MoveStrategyId {
    fn new_strat(self) -> Box<dyn MoveStrategy> {
        match self {
            MoveStrategyId::Human => Box::new(HumanMoveStrategy::default()),
            MoveStrategyId::Random => Box::new(RandomMoveStrategy::default()),
        }
    }
}

#[derive(Parser, Debug)]
pub struct CliArgs {
    #[arg(short = 'x', default_value_t = MoveStrategyId::Human)]
    x_strat: MoveStrategyId,
    #[arg(short = 'o', default_value_t = MoveStrategyId::Human)]
    o_strat: MoveStrategyId,
}

fn main() -> Result<()> {
    let args = CliArgs::parse();

    println!("Player X: {}", args.x_strat);
    println!("Player O: {}", args.o_strat);

    let mut game = Game::default();
    let mut player_x = args.x_strat.new_strat();
    let mut player_o = args.o_strat.new_strat();

    println!("\n{game}\n");
    while let GameState::Ongoing {
        next_player: cur_player,
    } = game.state
    {
        print!("Player {cur_player} enter your move: ");
        let strategy: &mut dyn MoveStrategy = match cur_player {
            PlayerId::X => player_x.as_mut(),
            PlayerId::O => player_o.as_mut(),
        };
        while let Err(err) = {
            let idx = strategy.get_move(&game.board);
            game.make_move(idx)
        } {
            println!("Invalid move; {err}; try again: ")
        }
        println!("\n{game}\n");
    }

    match game.state {
        GameState::Win { winner } => {
            println!("PLAYER {winner} WON!");
        }
        GameState::Tie => {
            println!("GAME IS A TIE!");
        }
        _ => unreachable!(),
    }

    Ok(())
}
