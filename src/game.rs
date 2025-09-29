use std::fmt::Display;

use smallvec::SmallVec;

use crate::{
    board::{Board, BoardIdx},
    player::{MoveStrategy, PlayerId},
};

#[derive(Copy, Clone, Debug)]
pub enum GameStatus {
    Ongoing,
    Tied,
    Won,
}

impl GameStatus {
    pub fn is_finished(self) -> bool {
        match self {
            GameStatus::Ongoing => false,
            GameStatus::Tied | GameStatus::Won => true,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct GameStateRef<'a, B> {
    pub cur_player: PlayerId,
    pub board: &'a B,
    pub turns: &'a [(PlayerId, BoardIdx)],
}

pub struct Game<B: Board, X: MoveStrategy, O: MoveStrategy> {
    pub board: B,
    pub x_strategy: X,
    pub o_strategy: O,
    pub cur_player: PlayerId,
    pub status: GameStatus,
    pub turns: SmallVec<[(PlayerId, BoardIdx); 9]>,
}

impl<B: Board, X: MoveStrategy, O: MoveStrategy> Game<B, X, O> {
    pub fn new(board: B, player_x: X, player_o: O) -> Self {
        Self {
            board,
            x_strategy: player_x,
            o_strategy: player_o,
            cur_player: PlayerId::X,
            status: GameStatus::Ongoing,
            turns: SmallVec::new(),
        }
    }

    pub fn advance(&mut self) -> GameStatus {
        if self.status.is_finished() {
            return self.status;
        }

        let state_ref = GameStateRef {
            cur_player: self.cur_player,
            board: &self.board,
            turns: &self.turns,
        };

        let move_idx = match self.cur_player {
            PlayerId::X => self.x_strategy.get_move(state_ref),
            PlayerId::O => self.o_strategy.get_move(state_ref),
        };

        self.board.set_unchecked(move_idx, self.cur_player);
        self.turns.push((self.cur_player, move_idx));

        self.status = if self.board.is_winner(self.cur_player) {
            GameStatus::Won
        } else if self.turns.len() == 9 {
            GameStatus::Tied
        } else {
            self.cur_player = self.cur_player.other();
            GameStatus::Ongoing
        };

        self.status
    }
}

impl<B: Board, X: MoveStrategy, O: MoveStrategy> Display for Game<B, X, O> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.board)?;

        match self.status {
            GameStatus::Ongoing => {
                writeln!(f, "Turn number: {}", self.turns.len())?;
                write!(f, "Next player: {}", self.cur_player)?;
            }
            GameStatus::Won => {
                write!(f, "PLAYER {} WON!", self.cur_player)?;
            }
            GameStatus::Tied => {
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
        fn get_move<B: Board>(&mut self, _: GameStateRef<'_, B>) -> BoardIdx {
            self.call_count.set(self.call_count.get() + 1);
            self.moves
                .borrow_mut()
                .pop_front()
                .expect("scripted strategy ran out of moves")
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
            GameStatus::Ongoing => assert_eq!(game.cur_player, PlayerId::O),
            other => panic!("expected ongoing status after first move, got {other:?}"),
        }
        assert_eq!(x_tracker.call_count(), 1);
        assert_eq!(o_tracker.call_count(), 0);

        match game.advance() {
            GameStatus::Ongoing => assert_eq!(game.cur_player, PlayerId::X),
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
                GameStatus::Ongoing => {}
                other => panic!("expected ongoing status before final winning move, got {other:?}"),
            }
        }

        match game.advance() {
            GameStatus::Won => assert_eq!(game.cur_player, PlayerId::X),
            other => panic!("expected X to win on fifth move, got {other:?}"),
        }

        let x_calls = x_tracker.call_count();
        let o_calls = o_tracker.call_count();

        match game.advance() {
            GameStatus::Won => assert_eq!(game.cur_player, PlayerId::X),
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
                GameStatus::Ongoing => {}
                other => panic!("expected ongoing status before O's winning move, got {other:?}"),
            }
        }

        match game.advance() {
            GameStatus::Won => assert_eq!(game.cur_player, PlayerId::O),
            other => panic!("expected O to win on sixth move, got {other:?}"),
        }

        let x_calls = x_tracker.call_count();
        let o_calls = o_tracker.call_count();

        match game.advance() {
            GameStatus::Won => assert_eq!(game.cur_player, PlayerId::O),
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
                GameStatus::Ongoing => {}
                other => panic!(
                    "expected ongoing status before final tie move (turn {turn}), got {other:?}"
                ),
            }
        }

        match game.advance() {
            GameStatus::Tied => {}
            other => panic!("expected tie after ninth move, got {other:?}"),
        }

        let x_calls = x_tracker.call_count();
        let o_calls = o_tracker.call_count();

        match game.advance() {
            GameStatus::Tied => {}
            other => panic!("expected tie status to persist after completion, got {other:?}"),
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
    fn game_status_is_finished_checks() {
        assert!(!GameStatus::Ongoing.is_finished());
        assert!(GameStatus::Won.is_finished());
        assert!(GameStatus::Tied.is_finished());
    }
}
