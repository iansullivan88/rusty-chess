pub mod commands;

use crate::{Color, Rank, Square, UnitKind};

#[derive(Copy, Clone, Debug)]
pub enum Move {
    Normal { from: Square, to: Square, captured_unit: Option<UnitKind> },
    KingSideCastle,
    QueenSideCastle,
    EnPassant { from: Square, to: Square },
    Promotion { from: Square, to: Square, promote_to: UnitKind, captured_unit: Option<UnitKind> }
}

pub struct MoveList {
    moves: [Move; 256],
    size: usize
}

impl MoveList {
    pub fn new() -> Self {
        Self {
            moves: [Move::KingSideCastle; 256],
            size: 0
        }
    }

    pub fn push(&mut self, new_move: Move) -> () {
        self.moves[self.size] = new_move;
        self.size+=1;
    }

    pub fn to_slice(&self) -> &[Move] {
        &self.moves[0..self.size]
    }
}

fn get_promotion_rank(color: Color) -> Rank {
    match color {
        Color::Black => Rank::One,
        Color::White => Rank::Eight
    }
}