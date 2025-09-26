use std::fmt::Display;

use crate::{MoveStrategy, board::Board, player::PlayerId};

#[derive(Copy, Clone, Debug)]
pub enum GameState {
    Ongoing { next_player: PlayerId },
    Won { winner: PlayerId },
    Tied,
}

impl GameState {
    pub fn is_finished(self) -> bool {
        match self {
            GameState::Ongoing { .. } => false,
            GameState::Won { .. } | GameState::Tied => true,
        }
    }
}

pub struct Game<B: Board, X: MoveStrategy, O: MoveStrategy> {
    board: B,
    player_x: X,
    player_o: O,
    state: GameState,
    turns: usize,
}

impl<B: Board, X: MoveStrategy, O: MoveStrategy> Game<B, X, O> {
    pub fn new(board: B, player_x: X, player_o: O) -> Self {
        Self {
            board,
            player_x,
            player_o,
            state: GameState::Ongoing {
                next_player: PlayerId::X,
            },
            turns: 0,
        }
    }

    pub fn advance(&mut self) -> GameState {
        let GameState::Ongoing {
            next_player: cur_player,
        } = self.state
        else {
            return self.state;
        };

        let move_idx = match cur_player {
            PlayerId::X => self.player_x.get_move(&self.board),
            PlayerId::O => self.player_o.get_move(&self.board),
        };

        self.board.set_unchecked(move_idx, cur_player);
        self.turns += 1;

        self.state = if self.board.is_winner(cur_player) {
            GameState::Won { winner: cur_player }
        } else if self.turns == 9 {
            GameState::Tied
        } else {
            GameState::Ongoing {
                next_player: cur_player.other(),
            }
        };

        self.state
    }
}

impl<B: Board, X: MoveStrategy, O: MoveStrategy> Display for Game<B, X, O> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.board)?;

        match self.state {
            GameState::Ongoing { next_player } => {
                writeln!(f, "Turn number: {}", self.turns)?;
                write!(f, "Next player: {next_player}")?;
            }
            GameState::Won { winner } => {
                write!(f, "PLAYER {winner} WON!")?;
            }
            GameState::Tied => {
                write!(f, "GAME TIED!")?;
            }
        }

        Ok(())
    }
}
