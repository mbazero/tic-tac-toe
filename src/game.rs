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

impl Default for GameState {
    fn default() -> Self {
        Self::Ongoing {
            next_player: PlayerId::default(),
        }
    }
}

pub struct Game<B: Board, X: MoveStrategy, O: MoveStrategy> {
    pub board: B,
    pub player_x: X,
    pub player_o: O,
    pub state: GameState,
    pub turns: usize,
}

impl<B: Board, X: MoveStrategy, O: MoveStrategy> Game<B, X, O> {
    pub fn new(board: B, player_x: X, player_o: O) -> Self {
        Self {
            board,
            player_x,
            player_o,
            state: GameState::default(),
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

    // TODO: Remove resets
    pub fn reset(&mut self) {
        self.board.reset();
        self.player_x.reset();
        self.player_o.reset();
        self.state = GameState::default();
        self.turns = 0;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        board::{Board, BoardIdx, array::ArrayBoard, bitset::BitsetBoard},
        player::PlayerId,
    };
    use rstest::rstest;
    use std::{
        cell::{Cell, RefCell},
        collections::VecDeque,
        rc::Rc,
    };

    trait BoardFactory {
        type Board: Board + Default;

        fn make(&self) -> Self::Board {
            Default::default()
        }
    }

    #[derive(Clone, Copy, Default)]
    struct ArrayBoardFactory;

    impl BoardFactory for ArrayBoardFactory {
        type Board = ArrayBoard;
    }

    #[derive(Clone, Copy, Default)]
    struct BitsetBoardFactory;

    impl BoardFactory for BitsetBoardFactory {
        type Board = BitsetBoard;
    }

    #[derive(Clone)]
    struct ScriptedStrategy {
        moves: Rc<RefCell<VecDeque<BoardIdx>>>,
        call_count: Rc<Cell<usize>>,
    }

    impl ScriptedStrategy {
        fn new(moves: impl Into<Vec<BoardIdx>>) -> Self {
            Self {
                moves: Rc::new(RefCell::new(VecDeque::from(moves.into()))),
                call_count: Rc::new(Cell::new(0)),
            }
        }

        fn call_count(&self) -> usize {
            self.call_count.get()
        }
    }

    impl MoveStrategy for ScriptedStrategy {
        fn get_move(&mut self, _board: &impl Board) -> BoardIdx {
            self.call_count.set(self.call_count.get() + 1);
            self.moves
                .borrow_mut()
                .pop_front()
                .expect("scripted strategy ran out of moves")
        }

        fn reset(&mut self) {
            panic!("cannot reset scripted strategy");
        }
    }

    fn make_game<F: BoardFactory>(
        factory: &F,
        x_moves: impl Into<Vec<BoardIdx>>,
        o_moves: impl Into<Vec<BoardIdx>>,
    ) -> (
        Game<F::Board, ScriptedStrategy, ScriptedStrategy>,
        ScriptedStrategy,
        ScriptedStrategy,
    ) {
        let x_strategy = ScriptedStrategy::new(x_moves);
        let o_strategy = ScriptedStrategy::new(o_moves);
        let x_tracker = x_strategy.clone();
        let o_tracker = o_strategy.clone();
        let game = Game::new(factory.make(), x_strategy, o_strategy);
        (game, x_tracker, o_tracker)
    }

    #[rstest]
    fn turn_progression(
        #[values(ArrayBoardFactory, BitsetBoardFactory)] factory: impl BoardFactory,
    ) {
        let (mut game, x_tracker, o_tracker) = make_game(&factory, vec![0, 2], vec![1, 3]);

        match game.advance() {
            GameState::Ongoing { next_player } => assert_eq!(next_player, PlayerId::O),
            other => panic!("expected ongoing state after first move, got {other:?}"),
        }
        assert_eq!(x_tracker.call_count(), 1);
        assert_eq!(o_tracker.call_count(), 0);

        match game.advance() {
            GameState::Ongoing { next_player } => assert_eq!(next_player, PlayerId::X),
            other => panic!("expected player X to be next, got {other:?}"),
        }
        assert_eq!(x_tracker.call_count(), 1);
        assert_eq!(o_tracker.call_count(), 1);
    }

    #[rstest]
    fn x_can_win(#[values(ArrayBoardFactory, BitsetBoardFactory)] factory: impl BoardFactory) {
        let (mut game, x_tracker, o_tracker) = make_game(&factory, vec![0, 1, 2], vec![3, 4]);

        for _ in 0..4 {
            match game.advance() {
                GameState::Ongoing { .. } => {}
                other => panic!("expected ongoing state before final winning move, got {other:?}"),
            }
        }

        match game.advance() {
            GameState::Won { winner } => assert_eq!(winner, PlayerId::X),
            other => panic!("expected X to win on fifth move, got {other:?}"),
        }

        let x_calls = x_tracker.call_count();
        let o_calls = o_tracker.call_count();

        match game.advance() {
            GameState::Won { winner } => assert_eq!(winner, PlayerId::X),
            other => panic!("expected game to remain won after completion, got {other:?}"),
        }
        assert_eq!(x_tracker.call_count(), x_calls);
        assert_eq!(o_tracker.call_count(), o_calls);
    }

    #[rstest]
    fn o_can_win(#[values(ArrayBoardFactory, BitsetBoardFactory)] factory: impl BoardFactory) {
        let (mut game, x_tracker, o_tracker) = make_game(&factory, vec![0, 8, 1], vec![3, 4, 5]);

        for _ in 0..5 {
            match game.advance() {
                GameState::Ongoing { .. } => {}
                other => panic!("expected ongoing state before O's winning move, got {other:?}"),
            }
        }

        match game.advance() {
            GameState::Won { winner } => assert_eq!(winner, PlayerId::O),
            other => panic!("expected O to win on sixth move, got {other:?}"),
        }

        let x_calls = x_tracker.call_count();
        let o_calls = o_tracker.call_count();

        match game.advance() {
            GameState::Won { winner } => assert_eq!(winner, PlayerId::O),
            other => panic!("expected game to remain won after completion, got {other:?}"),
        }
        assert_eq!(x_tracker.call_count(), x_calls);
        assert_eq!(o_tracker.call_count(), o_calls);
    }

    #[rstest]
    fn tie_detection(#[values(ArrayBoardFactory, BitsetBoardFactory)] factory: impl BoardFactory) {
        let (mut game, x_tracker, o_tracker) =
            make_game(&factory, vec![0, 2, 7, 3, 8], vec![4, 6, 1, 5]);

        for turn in 0..8 {
            match game.advance() {
                GameState::Ongoing { .. } => {}
                other => panic!(
                    "expected ongoing state before final tie move (turn {turn}), got {other:?}"
                ),
            }
        }

        match game.advance() {
            GameState::Tied => {}
            other => panic!("expected tie after ninth move, got {other:?}"),
        }

        let x_calls = x_tracker.call_count();
        let o_calls = o_tracker.call_count();

        match game.advance() {
            GameState::Tied => {}
            other => panic!("expected tie state to persist after completion, got {other:?}"),
        }
        assert_eq!(x_tracker.call_count(), x_calls);
        assert_eq!(o_tracker.call_count(), o_calls);
    }

    #[rstest]
    fn display_shows_ongoing_state(
        #[values(ArrayBoardFactory, BitsetBoardFactory)] factory: impl BoardFactory,
    ) {
        let (mut game, _, _) = make_game(&factory, vec![0, 3], vec![4]);
        game.advance();
        game.advance();

        let display = format!("{game}");
        assert!(display.contains("X| | "));
        assert!(display.contains(" |O| "));
        assert!(display.contains("Turn number: 2"));
        assert!(display.contains("Next player: X"));
    }

    #[rstest]
    fn display_shows_winner(
        #[values(ArrayBoardFactory, BitsetBoardFactory)] factory: impl BoardFactory,
    ) {
        let (mut game, _, _) = make_game(&factory, vec![0, 1, 2], vec![3, 4]);
        for _ in 0..5 {
            game.advance();
        }

        let display = format!("{game}");
        assert!(display.contains("X|X|X"));
        assert!(display.contains("PLAYER X WON!"));
    }

    #[rstest]
    fn display_shows_tie(
        #[values(ArrayBoardFactory, BitsetBoardFactory)] factory: impl BoardFactory,
    ) {
        let (mut game, _, _) = make_game(&factory, vec![0, 2, 7, 3, 8], vec![4, 6, 1, 5]);
        for _ in 0..9 {
            game.advance();
        }

        let display = format!("{game}");
        assert!(display.contains("GAME TIED!"));
    }

    #[test]
    fn game_state_is_finished_checks() {
        assert!(
            !GameState::Ongoing {
                next_player: PlayerId::X
            }
            .is_finished()
        );
        assert!(
            GameState::Won {
                winner: PlayerId::O
            }
            .is_finished()
        );
        assert!(GameState::Tied.is_finished());
    }
}
