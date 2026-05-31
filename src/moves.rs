pub mod commands;

use std::{ops::{Index, IndexMut}, path::Component::ParentDir, sync::LazyLock};

use crate::{ALL_FILES, ALL_RANKS, Board, Color::{self, Black}, File, Game, Rank, Square, Unit, UnitKind::{self, Pawn}, get_other_color, moves::Move::{EnPassant, KingSideCastle, QueenSideCastle}, utilities::StackVector};

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

const LEFT_RIGHT_OFFSETS: [MoveOffset; 2] = [
    MoveOffset { file: 1, rank: 0 },
    MoveOffset { file: -1, rank: 0 }
];

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
    MoveOffset { file: -1, rank: 1 },
    MoveOffset { file: 1, rank: 1 }
];

const BLACK_PAWN_CAPTURE_OFFSETS: [MoveOffset; 2] = [
    MoveOffset { file: -1, rank: -1 },
    MoveOffset { file: 1, rank: -1 }
];

const BLACK_KING_ATTACKING_PAWN_OFFSETS: [MoveOffset; 2] = [
    MoveOffset { file: -1, rank: -1 },
    MoveOffset { file: 1, rank: -1 }
];

const WHITE_KING_ATTACKING_PAWN_OFFSETS: [MoveOffset; 2] = [
    MoveOffset { file: -1, rank: 1 },
    MoveOffset { file: 1, rank: 1 }
];

const WHITE_MOVE_ONE_FORWARD: MoveOffset = MoveOffset { file: 0, rank: 1 };
const BLACK_MOVE_ONE_FORWARD: MoveOffset = MoveOffset { file: 0, rank: -1 };

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

const PROMOTION_UNITS: [UnitKind; 4] = [
    UnitKind::Queen,
    UnitKind::Knight,
    UnitKind::Rook,
    UnitKind::Bishop
];

const SQUARES_BETWEEN_WHITE_KING_AND_QUEEN_SIDE_ROOK: [Square; 3] = [
    Square(File::B, Rank::One),
    Square(File::C, Rank::One),
    Square(File::D, Rank::One)
];

const SQUARES_BETWEEN_WHITE_KING_AND_KING_SIDE_ROOK: [Square; 2] = [
    Square(File::F, Rank::One),
    Square(File::G, Rank::One)
];

const SQUARES_BETWEEN_BLACK_KING_AND_QUEEN_SIDE_ROOK: [Square; 3] = [
    Square(File::B, Rank::Eight),
    Square(File::C, Rank::Eight),
    Square(File::D, Rank::Eight)
];

const SQUARES_BETWEEN_BLACK_KING_AND_KING_SIDE_ROOK: [Square; 2] = [
    Square(File::F, Rank::Eight),
    Square(File::G, Rank::Eight)
];

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Move {
    Normal { from: Square, to: Square, captured_unit: Option<UnitKind> },
    KingSideCastle,
    QueenSideCastle,
    EnPassant { from: Square, to: Square },
    Promotion { from: Square, to: Square, promote_to: UnitKind, captured_unit: Option<UnitKind> }
}

pub type MoveList = StackVector<Move, 256>;

fn get_target_squares_for_offsets(board: &Board, square: Square, offsets: &[MoveOffset], moving_piece_color: Color) -> impl Iterator<Item = Square> {
    offsets
        .iter()
        .filter_map(move |&o| add_offset(square, o))
        .filter(move |&destination_square| board[destination_square].is_none_or(|p| p.color != moving_piece_color))
}

fn get_target_squares_for_pawn_capture_offsets(board: &Board, square: Square, offsets: &[MoveOffset], moving_piece_color: Color) -> impl Iterator<Item = Square> {
    offsets
        .iter()
        .filter_map(move |&o| add_offset(square, o))
        .filter(move |&destination_square| board[destination_square].is_some_and(|p| p.color != moving_piece_color))
}

fn is_piece_at_offset(board: &Board, source: Square, offsets: &[MoveOffset], kind: UnitKind, color: Color) -> bool {
    offsets
        .iter()
        .filter_map(|&o| add_offset(source, o))
        .filter_map(|s| board[s])
        .any(|p| p.color == color && p.kind == kind)
}

fn get_target_squares_for_directions(board: &Board, square: Square, directions: &[Direction], moving_piece_color: Color) -> impl Iterator<Item = Square> {
    directions
        .iter()
        .flat_map(move |&direction| {
            let mut hit_piece = false;

            // move along a direction and stop when hitting a piece
            // include the piece in the result if it is an enemy piece (capture)
            RAY_LOOKUP[(square, direction)].to_slice()
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

fn is_next_piece_on_ray_of_kind(board: &Board, source: Square, directions: &[Direction], color: Color, kind1: UnitKind, kind2: UnitKind) -> bool {
    for &direction in directions {
        let first_piece_in_direction = RAY_LOOKUP[(source, direction)]
            .to_slice()
            .iter()
            .filter_map(|&p| board[p])
            .next();

        if let Some(first_piece_in_direction) = first_piece_in_direction {
            if (first_piece_in_direction.color == color && (first_piece_in_direction.kind == kind1 || first_piece_in_direction.kind == kind2)) {
                return true;
            }
        }
    }

    return false;
}

pub fn populate_pseudo_legal_moves_for_source_square(game: &Game, square: Square, moves: &mut MoveList) {
    let Some(Unit { color, kind }) = game.board[square] else {
        return;
    };

    if color != game.next_move {
        return;
    }

    // castling
    if kind == UnitKind::King {
        if (game.black_can_king_side_castle && color == Color::Black && squares_are_empty(&game.board, &SQUARES_BETWEEN_BLACK_KING_AND_KING_SIDE_ROOK)) ||
           (game.white_can_king_side_castle && color == Color::White && squares_are_empty(&game.board, &SQUARES_BETWEEN_WHITE_KING_AND_KING_SIDE_ROOK)) {
            moves.push(KingSideCastle);
        }
        if (game.black_can_queen_side_castle && color == Color::Black && squares_are_empty(&game.board, &SQUARES_BETWEEN_BLACK_KING_AND_QUEEN_SIDE_ROOK)) ||
           (game.white_can_queen_side_castle && color == Color::White && squares_are_empty(&game.board, &SQUARES_BETWEEN_WHITE_KING_AND_QUEEN_SIDE_ROOK)) {
            moves.push(QueenSideCastle);
        }
    }

    let mut destination_squares: StackVector<Square, 27> = StackVector::new(Square(File::A, Rank::One));

    match kind {
        UnitKind::Pawn => {
            // en passant
            if let Some(en_passant_square) = game.en_passant_square {
                if en_passant_square.1 == square.1 && en_passant_square.0.idx().abs_diff(square.0.idx()) == 1 {
                    let rank_move_offset: i32 = if color == Color::White { 1 } else { -1 };
                    let destination_file = en_passant_square.0;
                    let destination_rank = ALL_RANKS[(square.1.idx() as i32 + rank_move_offset) as usize];
                    moves.push(EnPassant { from: square, to: Square(destination_file, destination_rank) });
                }
            }

            // pawn captures
            let capture_offsets = if color == Color::White {
                &WHITE_PAWN_CAPTURE_OFFSETS
            } else {
                &BLACK_PAWN_CAPTURE_OFFSETS
            };

            destination_squares.extend(get_target_squares_for_pawn_capture_offsets(&game.board, square, capture_offsets, game.next_move));

            // pawn moves
            let move_offset = if color == Color::White { WHITE_MOVE_ONE_FORWARD } else { BLACK_MOVE_ONE_FORWARD };
            let one_space_forward = add_offset(square, move_offset);
            if let Some(one_space_forward) = one_space_forward && game.board[one_space_forward].is_none() {
                // single move
                destination_squares.push(one_space_forward);

                let is_starting_pawn = (color == Color::White && square.1 == Rank::Two)
                    || (color == Color::Black && square.1 == Rank::Seven);

                // two space move
                if is_starting_pawn {
                    if let Some(two_spaces_forward) = add_offset(one_space_forward, move_offset) && game.board[two_spaces_forward].is_none() {
                        destination_squares.push(two_spaces_forward);
                    }
                }
            }
        }
        UnitKind::King => {
            destination_squares.extend(get_target_squares_for_offsets(&game.board, square, &KING_MOVE_OFFSETS, game.next_move));
        }
        UnitKind::Knight => {
            destination_squares.extend(get_target_squares_for_offsets(&game.board, square, &KNIGHT_MOVE_OFFSETS, game.next_move));
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

    for &destination_square in destination_squares.to_slice() {
        let captured_unit_kind = game.board[destination_square].map(|u| u.kind);
        if kind == UnitKind::Pawn && destination_square.1 == get_promotion_rank(color) {
            for promotion_unit in PROMOTION_UNITS {
                moves.push(Move::Promotion { from: square, to: destination_square, promote_to: promotion_unit, captured_unit: captured_unit_kind });
            }
        } else {
            // no promotion
            moves.push(Move::Normal { from: square, to: destination_square, captured_unit: captured_unit_kind });
        }
    }
}

fn squares_are_empty(board: &Board, squares: &[Square]) -> bool {
    squares.iter().all(|s| board[s].is_none())
}

fn get_promotion_rank(color: Color) -> Rank {
    match color {
        Color::Black => Rank::One,
        Color::White => Rank::Eight
    }
}

fn get_back_rank(color: Color) -> Rank {
    match color {
        Color::Black => Rank::Eight,
        Color::White => Rank::One
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

pub fn is_legal_move(board: &mut Board, king_position: Square, color: Color, pseudo_legal_move: Move) -> bool {
    apply_move_to_board(board, color, pseudo_legal_move);

    let other_color = get_other_color(color);
    let mut is_king_checked = false;
    
    // is king checked by knight
    is_king_checked = is_king_checked || is_piece_at_offset(board, king_position, &KNIGHT_MOVE_OFFSETS, UnitKind::Knight, other_color);

    // is king checked by pawn
    let attacking_pawn_offsets = if color == Color::White { &WHITE_KING_ATTACKING_PAWN_OFFSETS } else { &BLACK_KING_ATTACKING_PAWN_OFFSETS };
    is_king_checked = is_king_checked || is_piece_at_offset(board, king_position, attacking_pawn_offsets, UnitKind::Pawn, other_color);

    // is king checked by a diagonal moving piece
    is_king_checked = is_king_checked || is_next_piece_on_ray_of_kind(board, king_position, &BISHOP_DIRECTIONS, other_color, UnitKind::Bishop, UnitKind::Queen);
    
    // is king checked by a horizontal moving piece
    is_king_checked = is_king_checked || is_next_piece_on_ray_of_kind(board, king_position, &ROOK_DIRECTIONS, other_color, UnitKind::Rook, UnitKind::Queen);

    revert_move_to_board(board, color, pseudo_legal_move);

    !is_king_checked
}

pub fn apply_move_to_game(game: &mut Game, r#move: Move) {
    let new_king_square = match r#move {
        KingSideCastle => match game.next_move {
            Color::White => Some(Square(File::C, Rank::One)),
            Color::Black => Some(Square(File::C, Rank::Eight))
        },
        QueenSideCastle => match game.next_move {
            Color::White => Some(Square(File::C, Rank::One)),
            Color::Black => Some(Square(File::C, Rank::Eight))
        },
        Move::Normal { from, to , .. } if game.board[from].is_some_and(|u| u.kind == UnitKind::King) => Some(to),
        _ => None
    };

    // update state for king moves
    if let Some(new_king_square) = new_king_square  {
        if game.next_move == Color::White {
            game.white_king_position = new_king_square;
            game.white_can_king_side_castle = false;
            game.white_can_queen_side_castle = false;
        } else {
            game.black_king_position = new_king_square;
            game.black_can_king_side_castle = false;
            game.black_can_queen_side_castle = false;
        }
    }

    if let Move::Normal { from, to, .. } = r#move {
        // update state for rook moves
        if from == Square(File::A, Rank::One) {
            game.white_can_queen_side_castle = false;
        } else if from == Square(File::H, Rank::One) {
            game.white_can_king_side_castle = false;
        } else if from == Square(File::A, Rank::Eight) {
            game.black_can_queen_side_castle = false;
        } else if from == Square(File::H, Rank::Eight) {
            game.black_can_king_side_castle = false;
        }

        // update en passant state
        let is_double_pawn_move = from.0 == to.0 
            && from.1.idx().abs_diff(to.1.idx()) == 2
            && game.board[from].is_some_and(|u| u.kind == UnitKind::Pawn);
        if is_double_pawn_move {
            game.en_passant_square = Some(to);
        } else {
            game.en_passant_square = None;
        }
    }

    apply_move_to_board(&mut game.board, game.next_move, r#move);

    game.next_move = get_other_color(game.next_move);
}

pub fn apply_move_to_board(board: &mut Board, color: Color, r#move: Move) {
    match r#move {
        QueenSideCastle => {
            let rank = get_back_rank(color);
            // move the king
            board[Square(File::C, rank)] = board[Square(File::E, rank)];
            board[Square(File::E, rank)] = None;
            // move the rook
            board[Square(File::D, rank)] = board[Square(File::A, rank)];
            board[Square(File::A, rank)] = None;

        }
        KingSideCastle => {
            let rank = get_back_rank(color);
            // move the king
            board[Square(File::G, rank)] = board[Square(File::E, rank)];
            board[Square(File::E, rank)] = None;
            // move the rook
            board[Square(File::F, rank)] = board[Square(File::H, rank)];
            board[Square(File::H, rank)] = None;
        }
        Move::Normal { from, to, .. } => {
            board[to] = board[from];
            board[from] = None;
        }
        Move::Promotion { from, to, promote_to, .. } => {
            board[from] = None;
            board[to] = Some(Unit { color: color, kind: promote_to });
        }
        EnPassant { from, to } => {
            let capture_square = Square(to.0, from.1);
            board[to] = board[from];
            board[from] = None;
            board[capture_square] = None;
        }
    }
}

fn revert_move_to_board(board: &mut Board, color: Color, r#move: Move) {
    match r#move {
        QueenSideCastle => {
            let rank = get_back_rank(color);
            // move the king
            board[Square(File::E, rank)] = board[Square(File::C, rank)];
            board[Square(File::C, rank)] = None;
            // move the rook
            board[Square(File::A, rank)] = board[Square(File::D, rank)];
            board[Square(File::D, rank)] = None;

        }
        KingSideCastle => {
            let rank = get_back_rank(color);
            // move the king
            board[Square(File::E, rank)] = board[Square(File::G, rank)];
            board[Square(File::G, rank)] = None;
            // move the rook
            board[Square(File::H, rank)] = board[Square(File::F, rank)];
            board[Square(File::F, rank)] = None;
        }
        Move::Normal { from, to, captured_unit } => {
            board[from] = board[to];
            board[to] = captured_unit.map(|k| Unit { kind: k, color: get_other_color(color)});
        }
        Move::Promotion { from, to, captured_unit, .. } => {
            board[from] = Some(Unit { color: color, kind: UnitKind::Pawn });
            board[to] = captured_unit.map(|k| Unit { kind: k, color: get_other_color(color)});
        }
        EnPassant { from, to } => {
            let capture_square = Square(to.0, from.1);
            board[from] = board[to];
            board[to] = None;
            board[capture_square] = Some(Unit { color: get_other_color(color), kind: UnitKind::Pawn });
        }
    }
}

pub fn get_legal_moves(game: &mut Game) -> MoveList {
    let mut pseudo_legal_moves = MoveList::new(Move::KingSideCastle);
    for file in ALL_FILES {
        for rank in ALL_RANKS {
            populate_pseudo_legal_moves_for_source_square(game, Square(file, rank), &mut pseudo_legal_moves);
        }
    }

    let king_position = if game.next_move == Color::White { game.white_king_position } else { game.black_king_position };

    let mut legal_moves = MoveList::new(Move::KingSideCastle);
    let legal_move_iterator = pseudo_legal_moves
        .to_slice()
        .iter()
        .copied()
        .filter(|&m| is_legal_move(&mut game.board, king_position, game.next_move, m));

    for legal_move in legal_move_iterator {
        legal_moves.push(legal_move);
    }

    legal_moves
}
