
use crate::{ALL_FILES, ALL_RANKS, Board, Color, File, Game, Rank, Square, UnitKind, moves::{Move, MoveList, get_promotion_rank, is_legal_move, populate_pseudo_legal_moves_for_source_square}};

use std::{collections::HashSet, sync::LazyLock};
use regex::{Match, Regex};

static PIECE_MOVE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^(?<piece>[KQNBR])?(?<source_file>[a-h])?(?<source_rank>[1-8])?(?<file>[a-h])(?<rank>[1-8])(=(?<promotion>[KQNBR]))?$").unwrap()
});

#[derive(Copy, Clone, Debug)]
pub enum MoveCommand {
    KingSideCastle,
    QueenSideCastle,
    StandardMoveCommand { 
        unit_kind: UnitKind,
        destination: Square,
        source_rank: Option<Rank>,
        source_file: Option<File>,
        promotion: Option<UnitKind>
    }
}

pub fn parse_move(input: &str, game: &mut Game) -> Result<Move, String> {
    let move_command = parse_move_command(input)?;

    // a move command can be ambiguous eg NA2 - could refer to a move by more than one knight
    // get a list of possible moves this command could refer to
    let possible_moves_for_command = match move_command {
        MoveCommand::KingSideCastle => vec![Move::KingSideCastle],
        MoveCommand::QueenSideCastle => vec![Move::QueenSideCastle],
        MoveCommand::StandardMoveCommand { unit_kind, destination, source_rank, source_file, promotion } => {
            generate_possible_standard_moves_for_command(&game.board,game.next_move, source_rank, source_file, unit_kind, destination, promotion)?
        }
    };

    let moving_unit_kind = match move_command {
        MoveCommand::KingSideCastle => UnitKind::King,
        MoveCommand::QueenSideCastle => UnitKind::King,
        MoveCommand::StandardMoveCommand { unit_kind, .. } => unit_kind
    };

    let mut pseudo_legal_moves_for_unit_kind = MoveList::new(Move::KingSideCastle);

    for &rank in ALL_RANKS.iter() {
        for &file in ALL_FILES.iter() {
            if let Some(unit) = game.board[Square(file, rank)] && unit.kind == moving_unit_kind && unit.color == game.next_move {
                populate_pseudo_legal_moves_for_source_square(game, Square(file, rank), &mut pseudo_legal_moves_for_unit_kind)
            }
        }
    }

    let pseudo_legal_moves_for_unit_kind: HashSet<_> = pseudo_legal_moves_for_unit_kind
        .to_slice()
        .into_iter()
        .collect();

    let possible_pseudo_legal_moves_for_command: Vec<_> = possible_moves_for_command
        .into_iter()
        .filter(|m| pseudo_legal_moves_for_unit_kind.contains(m))
        .collect();


    if possible_pseudo_legal_moves_for_command.len() == 0 {
        return Err(String::from("Not a legal move"))
    }

    if possible_pseudo_legal_moves_for_command.len() > 1 {
        return Err(String::from("Ambiguous move, specify rank and/or file"))
    }

    let only_pseudo_legal_move = possible_pseudo_legal_moves_for_command[0];

    let king_position = if game.next_move == Color::White { game.white_king_position } else { game.black_king_position };
    
    if !is_legal_move(&mut game.board, king_position, game.next_move, only_pseudo_legal_move) {
        return Err(String::from("Cannot leave own king in check"))
    }

    return Ok(only_pseudo_legal_move);

}

fn generate_possible_standard_moves_for_command(board: &Board, color: Color, source_rank: Option<Rank>, source_file: Option<File>, unit_kind: UnitKind, destination: Square, promotion: Option<UnitKind>) -> Result<Vec<Move>, String> {    
    if promotion.is_some() {
        if unit_kind == UnitKind::Pawn {
            if destination.1 != get_promotion_rank(color) {
                return Err(String::from("Cannot promote before the last rank"))
            }
        } else {
            return Err(String::from("Only pawns may promote"))
        }
    }
    
    let destination_unit = board[destination];
    let captured_unit_kind = match destination_unit {
        None => None,
        Some(unit) => if unit.color == color {
            return Err(String::from("Cannot capture own piece"))
        } else {
            Some(unit.kind)
        }
    };

    let possible_source_ranks: &[Rank] = match source_rank {
        None => &ALL_RANKS,
        Some(rank) => &[rank]
    };

    let possible_source_files: &[File] = match source_file {
        None => &ALL_FILES,
        Some(file) => &[file]
    };

    let mut matching_sources: Vec<Square> = Vec::new();
    for rank in possible_source_ranks.iter().copied() {
        for file in possible_source_files.iter().copied() {
            if let Some(unit) = board[Square(file, rank)] {
                if unit.color == color && unit.kind == unit_kind  {
                    matching_sources.push(Square(file, rank));
                }
            }
        }
    }

    let mut moves: Vec<Move> = Vec::new();

    for source in matching_sources {
        if unit_kind == UnitKind::Pawn {
            if let Some(promotion_kind) = promotion {
                moves.push(Move::Promotion { from: source, to: destination, promote_to: promotion_kind, captured_unit: captured_unit_kind });
                continue;
            }
            // a pawn move to a different file without capture must be en passant
            else if destination_unit.is_none() && source.0 != destination.0 {
                moves.push(Move::EnPassant { from: source, to: destination });
                continue;
            }
        }

        moves.push(Move::Normal { from: source, to: destination, captured_unit: captured_unit_kind });
    }

    Ok(moves)
}

fn parse_move_command(input: &str) -> Result<MoveCommand, String> {
    let input = input.trim();

    if input.eq_ignore_ascii_case("o-o") {
        return Ok(MoveCommand::KingSideCastle)
    }

    if input.eq_ignore_ascii_case("o-o-o") {
        return Ok(MoveCommand::QueenSideCastle)
    }
    
    let Some(caps) = PIECE_MOVE_REGEX.captures(input) else {
        return Err(String::from("Not a valid command"))
    };

    let unit_kind = match caps.name("piece") {
        None => UnitKind::Pawn,
        Some(cap_match) => { 
            let upper = cap_match.as_str().to_uppercase();
            match upper.as_str() {
                "B" => UnitKind::Bishop,
                "K" => UnitKind::King,
                "N" => UnitKind::Knight,
                "R" => UnitKind::Rook,
                "Q" => UnitKind::Queen,
                otherwise => return Err(format!("Invalid unit kind {}", otherwise)) 
            }
        }
    };

    fn match_to_rank(rank_match: Match) -> Rank {
        parse_rank(rank_match.as_str()).expect("Invalid rank string")
    }

    fn match_to_file(file_match: Match) -> File {
        parse_file(file_match.as_str()).expect("Invalid file string")
    }

    fn match_to_unit(file_match: Match) -> UnitKind {
        parse_piece_kind(file_match.as_str()).expect("Invalid unit string")
    }

    Ok(MoveCommand::StandardMoveCommand {
        unit_kind,
        destination: Square(
            match_to_file(caps.name("file").expect("File not matched by regex")),
            match_to_rank(caps.name("rank").expect("Rank not matched by regex"))
        ),
        source_rank: caps.name("source_rank").map(|r| match_to_rank(r)),
        source_file: caps.name("source_file").map(|f| match_to_file(f)),
        promotion: caps.name("promotion").map(|p| match_to_unit(p)),
    })
}

fn parse_rank(input: &str) -> Option<Rank> {
    match input {
        "1" => Some(Rank::One),
        "2" => Some(Rank::Two),
        "3" => Some(Rank::Three),
        "4" => Some(Rank::Four),
        "5" => Some(Rank::Five),
        "6" => Some(Rank::Six),
        "7" => Some(Rank::Seven),
        "8" => Some(Rank::Eight),
        _ => None
    }
}

fn parse_file(input: &str) -> Option<File> {
    let lower = input.to_lowercase();
    match lower.as_str() {
        "a" => Some(File::A),
        "b" => Some(File::B),
        "c" => Some(File::C),
        "d" => Some(File::D),
        "e" => Some(File::E),
        "f" => Some(File::F),
        "g" => Some(File::G),
        "h" => Some(File::H),
        _ => None
    }
}

fn parse_piece_kind(input: &str) -> Option<UnitKind> {
    let kind = match input {
        "B" => UnitKind::Bishop,
        "K" => UnitKind::King,
        "N" => UnitKind::Knight,
        "R" => UnitKind::Rook,
        "Q" => UnitKind::Queen,
        _ => return None
    };

    Some(kind)
}