
use crate::{ALL_FILES, ALL_RANKS, Board, Color, File, Rank, Square, UnitKind, moves::{Move, get_promotion_rank}};

use std::sync::LazyLock;
use regex::{Match, Regex};

static PIECE_MOVE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?<piece>[KQNBR])?(?<source_file>[a-h])?(?<source_rank>[1-8])?(?<file>[a-h])(?<rank>[1-8])(=(?<promotion>[KQNBR]))?$").unwrap()
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

    pub fn parse_move(input: &str, board: &Board, color: Color) -> Result<Vec<Move>, String> {
    let move_command = parse_move_command(input)?;
    to_moves(move_command, board, color)
}

fn to_moves(command: MoveCommand, board: &Board, color: Color) -> Result<Vec<Move>, String> {
    match command {
        MoveCommand::KingSideCastle => Ok(vec![Move::KingSideCastle]),
        MoveCommand::QueenSideCastle => Ok(vec![Move::QueenSideCastle]),
        MoveCommand::StandardMoveCommand{ source_rank, source_file, unit_kind , destination, promotion } => {      
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
    }
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
        Some(cap_match) => match cap_match.as_str() {
            "B" => UnitKind::Bishop,
            "K" => UnitKind::King,
            "N" => UnitKind::Knight,
            "R" => UnitKind::Rook,
            "Q" => UnitKind::Queen,
            otherwise => return Err(format!("Invalid unit kind {}", otherwise)) 
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
    match input {
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