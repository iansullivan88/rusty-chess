use std::ops::{Index, IndexMut};

pub mod moves;
pub mod utilities;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Color {
    White, Black
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum UnitKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum File {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H
}

impl File {
    fn idx(self) -> usize {
        self as usize
    }
}

pub const ALL_FILES: [File; 8] = [
    File::A,
    File::B,
    File::C, 
    File::D,
    File::E,
    File::F,
    File::G,
    File::H
];

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Rank {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight
}

impl Rank {
    fn idx(self) -> usize {
        self as usize
    }
}

pub const ALL_RANKS: [Rank; 8] = [
    Rank::One,
    Rank::Two,
    Rank::Three, 
    Rank::Four,
    Rank::Five,
    Rank::Six,
    Rank::Seven,
    Rank::Eight
];

#[derive(Copy, Clone, Debug)]
pub struct Square(File, Rank);

#[derive(Copy, Clone, Debug)]
pub struct Unit {
    pub kind: UnitKind,
    pub color: Color
}

pub struct Board {
    pub squares: [[Option<Unit>; 8]; 8]
}

impl Index<Square> for Board {
    type Output = Option<Unit>;

    fn index(&self, index: Square) -> &Self::Output {
        let Square(file, rank) = index;
        &self.squares[(7)-rank.idx()][file.idx()]
    }
}

impl IndexMut<Square> for Board {
    fn index_mut(&mut self, index: Square) -> &mut Self::Output {
        let Square(file, rank) = index;
        &mut self.squares[(7)-rank.idx()][file.idx()]
    }
}

pub struct Game {
    pub board: Board,
    pub next_move: Color,
    pub white_can_king_side_castle: bool,
    pub black_can_king_side_castle: bool,
    pub white_can_queen_side_castle: bool,
    pub black_can_queen_side_castle: bool,
    pub en_passant_square: Option<Square>
}

impl Game {
    pub fn new() -> Self {

        fn create_row(color: Color, units: [UnitKind; 8]) -> [Option<Unit>; 8] {   
            units.map(|u| Some(Unit {
                kind: u,
                color: color
            }))
        }

        let pieces= [UnitKind::Rook, UnitKind::Knight, UnitKind::Bishop, UnitKind::Queen, UnitKind::King, UnitKind::Bishop, UnitKind::Knight, UnitKind::Rook];

        Self {
            next_move: Color::White,
            white_can_king_side_castle: true,
            black_can_king_side_castle: true,
            white_can_queen_side_castle: true,
            black_can_queen_side_castle: true,
            en_passant_square: None,
            board: Board {
                squares: [
                    create_row(Color::Black, pieces),
                    create_row(Color::Black, [UnitKind::Pawn; 8]),
                    [None::<Unit>; 8],
                    [None::<Unit>; 8],
                    [None::<Unit>; 8],
                    [None::<Unit>; 8],
                    create_row(Color::White, [UnitKind::Pawn; 8]),
                    create_row(Color::White, pieces)
                ]
            }
        }
    }
}