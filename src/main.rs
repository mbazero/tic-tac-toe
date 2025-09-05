use anyhow::{Context, Result, bail};
use regex::Regex;
use std::{
    fmt::Display,
    io::{self, Write},
    str::FromStr,
    sync::LazyLock,
};

pub struct Game {
    state: GameState,
    board: [Option<Player>; 9],
    moves: usize,
}

impl Game {
    pub fn make_move(&mut self, coords: Coords) -> Result<()> {
        let GameState::Ongoing {
            next_player: cur_player,
        } = self.state
        else {
            return Ok(());
        };

        let idx = coords.i * 3 + coords.j;
        let Some(cell) = self.board.get_mut(idx) else {
            bail!("cell is out of bounds: {coords:?}");
        };
        if cell.is_some() {
            bail!("cell is occupied: {coords:?}");
        }
        *cell = Some(cur_player);
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
                    Player::X => Player::O,
                    Player::O => Player::X,
                },
            }
        }

        Ok(())
    }

    pub fn print_board(&self) {}
}

impl Default for Game {
    fn default() -> Self {
        Self {
            state: GameState::Ongoing {
                next_player: Player::X,
            },
            board: [None; 9],
            moves: 0,
        }
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cc = |i: usize| match self.board[i] {
            Some(Player::X) => 'X',
            Some(Player::O) => 'O',
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
    Ongoing { next_player: Player },
    Win { winner: Player },
    Tie,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Player {
    X,
    O,
}

impl Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Player::X => 'X',
                Player::O => 'O',
            }
        )
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Coords {
    i: usize,
    j: usize,
}

impl FromStr for Coords {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        static RGX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d),(\d)$").unwrap());
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

fn main() -> Result<()> {
    let mut cli_reader = CliReader::default();
    let mut game = Game::default();

    println!("\n{game}\n");
    while let GameState::Ongoing {
        next_player: cur_player,
    } = game.state
    {
        print!("Player {cur_player} enter your move: ");
        while cli_reader
            .read::<Coords>()
            .and_then(|coords| game.make_move(coords))
            .is_err()
        {
            print!("Invalid move, try again: ");
        }
        println!("\n{game}\n");
    }

    game.print_board();
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
