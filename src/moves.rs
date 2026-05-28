pub mod commands;

use std::{iter, ops::{Index, IndexMut}, sync::LazyLock};

use crate::{ALL_FILES, ALL_RANKS, Board, Color, File, Game, Rank, Square, Unit, UnitKind, moves::Move::{KingSideCastle, QueenSideCastle}, utilities::StackVector};

#[derive(Copy, Clone)]
struct MoveOffset { file: i8, rank: i8 }

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Direction {
    // clockwise starting at 12
    Up,
    UpRight,
    Right,
    DownRight,
    Down,
    DownLeft,
    Left,
    Upleft
}

pub static RAY_LOOKUP: LazyLock<RayLookup> = LazyLock::new(|| {
    let directions_and_offsets = [
        (Direction::Up, MoveOffset { file: 0, rank: 1} ),
        (Direction::UpRight, MoveOffset { file: 1, rank: 1} ),
        (Direction::Right, MoveOffset { file: 1, rank: 0} ),
        (Direction::DownRight, MoveOffset { file: 1, rank: -1} ),
        (Direction::Down, MoveOffset { file: 0, rank: -1} ),
        (Direction::DownLeft, MoveOffset { file: -1, rank: -1} ),
        (Direction::Left, MoveOffset { file: -1, rank: 0} ),
        (Direction::Upleft, MoveOffset { file: 1, rank: -1} )
    ];

    let mut rays = vec![Option::None; 8 * 8 * 8];

    let empty_square = Square(File::A, Rank::One);

    for (direction, offset) in directions_and_offsets {
        for file in ALL_FILES {
            for rank in ALL_RANKS {
                let mut ray = Ray::new(empty_square);

                let mut current_square = Square(file, rank);
                // move in a direction, collecting squares
                loop {
                    if let Some(next_square) = add_offset(current_square, offset) {
                        ray.push(next_square);
                        current_square = next_square;
                    } else {
                        break;
                    }
                }

                let index = to_ray_lookup_index(Square(file, rank), direction);
                rays[index] = Some(ray);
            }
        }
    }

    let initialized_vector: Vec<Ray> = rays
        .into_iter()
        .map(|opt| opt.expect("Rays weren't initialized"))
        .collect();

    RayLookup {
        rays: initialized_vector.into_boxed_slice().try_into().unwrap()
    }
});

type Ray = StackVector<Square, 7>;

pub struct RayLookup {
    // one for each direction on every square
    pub rays: Box<[Ray; 8 * 64]>
}

impl Index<(Square, Direction)> for RayLookup {
    type Output = Ray;

    fn index(&self, index: (Square, Direction)) -> &Self::Output {
        let index = to_ray_lookup_index(index.0, index.1);
        &self.rays[index]
    }
}

impl IndexMut<(Square, Direction)> for RayLookup {
    fn index_mut(&mut self, index: (Square, Direction)) -> &mut Self::Output {
        let index = to_ray_lookup_index(index.0, index.1);
        &mut self.rays[index]
    }
}

fn to_ray_lookup_index(square: Square, direction: Direction) -> usize {
    let Square(file, rank) = square;
    (direction as usize) * 8 * 8 + (file as usize) * 8 + (rank as usize)
}

const KING_MOVE_OFFSETS: [MoveOffset; 8] = [
    // clockwise, starting at midnight
    MoveOffset { file: 0, rank: 1 },
    MoveOffset { file: 1, rank: 1 },
    MoveOffset { file: 1, rank: 0 },
    MoveOffset { file: 1, rank: -1 },
    MoveOffset { file: 0, rank: -1 },
    MoveOffset { file: -1, rank: -1 },
    MoveOffset { file: -1, rank: 0 },
    MoveOffset { file: -1, rank: 1 }
];

const KNIGHT_MOVE_OFFSETS: [MoveOffset; 8] = [
    MoveOffset { file: -1, rank: 2 },
    MoveOffset { file: 1, rank: 2 },
    MoveOffset { file: 2, rank: -1 },
    MoveOffset { file: 2, rank: 1 },
    MoveOffset { file: 1, rank: -2 },
    MoveOffset { file: -1, rank: -2 },
    MoveOffset { file: -2, rank: -1 },
    MoveOffset { file: -2, rank: 1 }
];

const WHITE_PAWN_CAPTURE_OFFSETS: [MoveOffset; 2] = [
    MoveOffset { file: 1, rank: 1 },
    MoveOffset { file: 1, rank: -1 }
];

const BLACK_PAWN_CAPTURE_OFFSETS: [MoveOffset; 2] = [
    MoveOffset { file: 1, rank: 1 },
    MoveOffset { file: 1, rank: -1 }
];

const STARTING_WHITE_PAWN_OFFSETS: [MoveOffset; 2] = [
    MoveOffset { file: 0, rank: 1 },
    MoveOffset { file: 0, rank: 2 }
];

const STARTING_BLACK_PAWN_OFFSETS: [MoveOffset; 2] = [
    MoveOffset { file: 0, rank: -1 },
    MoveOffset { file: 0, rank: -2 }
];

const WHITE_PAWN_OFFSETS: [MoveOffset; 1] = [
    MoveOffset { file: 0, rank: 1 }
];

const BLACK_PAWN_OFFSETS: [MoveOffset; 1] = [
    MoveOffset { file: 0, rank: -1 }
];

const BISHOP_DIRECTIONS: [Direction; 4] = [
    Direction::UpRight,
    Direction::DownRight,
    Direction::DownLeft,
    Direction::Upleft
];

const ROOK_DIRECTIONS: [Direction; 4] = [
    Direction::Up,
    Direction::Right,
    Direction::Down,
    Direction::Left
];

const QUEEN_DIRECTIONS: [Direction; 8] = [
    Direction::Up,
    Direction::Right,
    Direction::Down,
    Direction::Left,
    Direction::UpRight,
    Direction::DownRight,
    Direction::DownLeft,
    Direction::Upleft
];

#[derive(Copy, Clone, Debug)]
pub enum Move {
    Normal { from: Square, to: Square, captured_unit: Option<UnitKind> },
    KingSideCastle,
    QueenSideCastle,
    EnPassant { from: Square, to: Square },
    Promotion { from: Square, to: Square, promote_to: UnitKind, captured_unit: Option<UnitKind> }
}

type MoveList = StackVector<Move, 256>;
type SquareList = StackVector<Move, 22>;

fn get_target_squares_for_offsets(board: &Board, square: Square, offsets: &[MoveOffset], moving_piece_color: Color, allow_capture: bool) -> impl Iterator<Item = Square> {
    offsets
        .iter()
        .filter_map(move |&o| add_offset(square, o))
        .filter(move |&destination_square| board[destination_square].is_none_or(|p| allow_capture && p.color != moving_piece_color))
}

fn get_target_squares_for_pawn_capture_offsets(board: &Board, square: Square, offsets: &[MoveOffset], moving_piece_color: Color) -> impl Iterator<Item = Square> {
    offsets
        .iter()
        .filter_map(move |&o| add_offset(square, o))
        .filter(move |&destination_square| board[destination_square].is_some_and(|p| p.color != moving_piece_color))
}

fn get_target_squares_for_directions(board: &Board, square: Square, directions: &[Direction], moving_piece_color: Color) -> impl Iterator<Item = Square> {
    directions
        .iter()
        .flat_map(move |direction| {
            let mut hit_piece = false;

            // move along a direction and stop when hitting a piece
            // include the piece in the result if it is an enemy piece (capture)
            RAY_LOOKUP[(square, *direction)].to_slice()
                .iter()
                .copied()
                .filter_map(move |destination_square| {
                    if hit_piece {
                        return None
                    }

                    let Some(unit) = board[destination_square] else {
                        return Some(destination_square)
                    };

                    hit_piece = true;

                    if unit.color == moving_piece_color {
                        None
                    } else {
                        Some(destination_square)
                    }
                })
        })
}

fn populate_pseudo_legal_moves_for_source_square(game: &Game, square: Square, moves: &mut MoveList) {
    let Some(Unit { color, kind }) = game.board[square] else {
        return;
    };

    // castling
    if kind == UnitKind::King {
        if (game.black_can_king_side_castle && color == Color::Black) ||
           (game.white_can_king_side_castle && color == Color::White) {
            moves.push(KingSideCastle);
        }
        if (game.black_can_queen_side_castle && color == Color::Black) ||
           (game.white_can_queen_side_castle && color == Color::White) {
            moves.push(QueenSideCastle);
        }
    }

    let mut destination_squares: StackVector<Square, 27> = StackVector::new(Square(File::A, Rank::One));
    
    match kind {
        UnitKind::Pawn => {
            // en passant (TODO)
            if let Some(en_passant_square) = game.en_passant_square {

            }

            // pawn captures (TODO)
            let capture_offsets = if color == Color::White {
                &WHITE_PAWN_CAPTURE_OFFSETS
            } else {
                &BLACK_PAWN_CAPTURE_OFFSETS
            };

            destination_squares.extend(get_target_squares_for_pawn_capture_offsets(&game.board, square, capture_offsets, game.next_move));

            // pawn moves
            let offsets: &[MoveOffset] = match (square.1, color) {
                (Rank::Two, Color::White) => &STARTING_WHITE_PAWN_OFFSETS,
                (Rank::Seven, Color::Black) => &STARTING_BLACK_PAWN_OFFSETS,
                (_, Color::White) => &WHITE_PAWN_OFFSETS,
                (_, Color::Black) => &BLACK_PAWN_OFFSETS,
            };

            destination_squares.extend(get_target_squares_for_offsets(&game.board, square, offsets, game.next_move, false));
        }
        UnitKind::King => {
            destination_squares.extend(get_target_squares_for_offsets(&game.board, square, &KING_MOVE_OFFSETS, game.next_move, true));
        }
        UnitKind::Knight => {
            destination_squares.extend(get_target_squares_for_offsets(&game.board, square, &KNIGHT_MOVE_OFFSETS, game.next_move, true));
        }
        UnitKind::Bishop => {
            destination_squares.extend(get_target_squares_for_directions(&game.board, square, &BISHOP_DIRECTIONS, game.next_move));
        }
        UnitKind::Rook => {
            destination_squares.extend(get_target_squares_for_directions(&game.board, square, &ROOK_DIRECTIONS, game.next_move));
        }
        UnitKind::Queen => {
            destination_squares.extend(get_target_squares_for_directions(&game.board, square, &QUEEN_DIRECTIONS, game.next_move));
        }
    }
}

fn get_promotion_rank(color: Color) -> Rank {
    match color {
        Color::Black => Rank::One,
        Color::White => Rank::Eight
    }
}

fn add_offset(square: Square, offset: MoveOffset) -> Option<Square> {
    let new_file_idx = (square.0.idx() as i8) + offset.file;
    let new_rank_idx = (square.1.idx() as i8) + offset.rank;

    if new_file_idx >= 0
        && new_file_idx < 8
        && new_rank_idx >= 0
        && new_rank_idx < 8 {
            let square = Square(ALL_FILES[new_file_idx as usize], ALL_RANKS[new_rank_idx as usize]);
            Some(square)
        } else {
            None
        }
}