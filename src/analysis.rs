use crate::{ALL_FILES, ALL_RANKS, Board, Color, Game, Square, Unit, UnitKind, moves::{Move, get_legal_moves, use_modified_board}, utilities::NonNanFloat};
use rand::{Rng};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Algorithm {
    Random,
    NextBestMove
}

pub fn choose_move(game: &mut Game, algorithm: Algorithm) -> Move {
    match algorithm {
        Algorithm::Random => choose_random_move(game),
        Algorithm::NextBestMove => choose_next_best_move(game)
    }
}

fn evaluate_move(game: &mut Game, r#move : Move) -> NonNanFloat {
    use_modified_board(&mut game.board, game.next_move, r#move, evaluate_board)
}

/// Return a score that represents the strength of the board
/// A value greater than 0 means white has the advantage
fn evaluate_board(board: &Board) -> NonNanFloat {
    
    let mut score = 0.0;

    for file in ALL_FILES {
        for rank in ALL_RANKS {
            if let Some(Unit { kind, color}) = board[Square(file, rank)] {
                let unit_score: f64 = match kind {
                    UnitKind::Bishop => 3.0,
                    UnitKind::King => 100.0,
                    UnitKind::Queen => 9.0,
                    UnitKind::Knight => 3.0,
                    UnitKind::Pawn => 1.0,
                    UnitKind::Rook => 5.0
                };

                if color == Color::White { score = score + unit_score } else { score = score - unit_score };
            }
        }
    }

    NonNanFloat::new(score)
}

fn choose_random_move(game: &mut Game) -> Move {

    let legal_moves = get_legal_moves(game);
    let legal_moves = legal_moves.to_slice().iter().copied().collect();

    *choose_random(&legal_moves)
        .expect("Method should not be called if there are no legal moves")
}

fn choose_next_best_move(game: &mut Game) -> Move {

    let legal_moves = get_legal_moves(game);

    let scored_moves: Vec<_> = legal_moves
        .to_slice()
        .iter()
        .copied()
        .map(|m| (m, evaluate_move(game, m)))
        .collect();

    let scores = scored_moves
        .iter()
        .map(|&(_, score)| score);

    let target_score = if game.next_move == Color::White {
        scores.max()
    } else {
        scores.min()
    };

    let target_score = target_score
        .expect("Method should not be called if there are no legal moves");

    let best_moves: Vec<_> = scored_moves
        .into_iter()
        .filter(|&(_, score)| score == target_score)
        .map(|(m, _)| m)
        .collect();

    *choose_random(&best_moves).unwrap()
}

fn choose_random<T>(items: &Vec<T>) -> Option<&T> {
    if items.len() == 0 {
        return None
    }

    let mut rng = rand::thread_rng();

    let random_index = rng.gen_range(0..items.len());

    Some(&items[random_index])
}