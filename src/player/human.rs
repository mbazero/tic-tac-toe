use anyhow::{Context, Result};
use regex::Regex;
use std::{
    io::{self, Write},
    str::FromStr,
    sync::LazyLock,
};

use crate::{
    board::{Board, BoardIdx},
    player::{MoveStrategy, PlayerId},
};

pub struct HumanMoveStrategy {
    player: PlayerId,
    cli_reader: CliReader,
}

impl HumanMoveStrategy {
    pub fn new(player: PlayerId) -> Self {
        Self {
            player,
            cli_reader: CliReader::default(),
        }
    }

    fn get_move_checked(&mut self, board: &impl Board) -> Result<BoardIdx> {
        let idx = self.cli_reader.read::<Coords>()?.to_board_idx();
        board.check_set(idx)?;
        Ok(idx)
    }
}

impl MoveStrategy for HumanMoveStrategy {
    fn get_move(&mut self, board: &impl Board) -> BoardIdx {
        loop {
            print!("Player {} enter your move: ", self.player);
            match self.get_move_checked(board) {
                Ok(idx) => return idx,
                Err(err) => {
                    println!("Invalid move: {err}");
                }
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct Coords {
    i: u8,
    j: u8,
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

impl Coords {
    fn to_board_idx(self) -> BoardIdx {
        self.i * 3 + self.j
    }
}

#[derive(Debug, Default)]
struct CliReader {
    buffer: String,
}

impl CliReader {
    pub fn read<T>(&mut self) -> Result<T>
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
