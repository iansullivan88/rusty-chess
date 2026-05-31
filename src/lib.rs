use std::{fmt, ops::{Index, IndexMut}};

pub mod moves;
pub mod utilities;
pub mod analysis;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Color {
    White, Black
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
         match self {
            Color::White => write!(f, "White"),
            Color::Black => write!(f, "Black")
         }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum UnitKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
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

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            File::A => write!(f, "A"),
            File::B => write!(f, "B"),
            File::C => write!(f, "C"),
            File::D => write!(f, "D"),
            File::E => write!(f, "E"),
            File::F => write!(f, "F"),
            File::G => write!(f, "G"),
            File::H => write!(f, "H")
        }
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
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

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.idx() + 1)
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Square(pub File, pub Rank);

#[derive(Copy, Clone, Debug)]
pub struct Unit {
    pub kind: UnitKind,
    pub color: Color
}

pub struct Board {
    pub squares: [Option<Unit>; 64]
}

impl Index<Square> for Board {
    type Output = Option<Unit>;

    fn index(&self, index: Square) -> &Self::Output {
        let Square(file, rank) = index;
        &self.squares[rank.idx() * 8 + file.idx()]
    }
}

impl Index<&Square> for Board {
    type Output = Option<Unit>;

    fn index(&self, index: &Square) -> &Self::Output {
        let Square(file, rank) = index;
        &self.squares[rank.idx() * 8 + file.idx()]
    }
}

impl IndexMut<Square> for Board {
    fn index_mut(&mut self, index: Square) -> &mut Self::Output {
        let Square(file, rank) = index;
        &mut self.squares[rank.idx() * 8 + file.idx()]
    }
}

impl IndexMut<&Square> for Board {
    fn index_mut(&mut self, index: &Square) -> &mut Self::Output {
        let Square(file, rank) = index;
        &mut self.squares[rank.idx() * 8 + file.idx()]
    }
}

pub struct Game {
    pub board: Board,
    pub next_move: Color,
    pub white_can_king_side_castle: bool,
    pub black_can_king_side_castle: bool,
    pub white_can_queen_side_castle: bool,
    pub black_can_queen_side_castle: bool,
    pub en_passant_square: Option<Square>,
    pub white_king_position: Square,
    pub black_king_position: Square
}

impl Game {
    pub fn new() -> Self {

        fn create_row(color: Color, units: [UnitKind; 8]) -> [Option<Unit>; 8] {   
            units.map(|u| Some(Unit {
                kind: u,
                color: color
            }))
        }

        fn set_row(board: &mut Board, rank: Rank, units: [Option<Unit>; 8]) {
            for (unit, file) in units.into_iter().zip(ALL_FILES) {
                board[Square(file, rank)] = unit;
            }
        }

        let pieces= [UnitKind::Rook, UnitKind::Knight, UnitKind::Bishop, UnitKind::Queen, UnitKind::King, UnitKind::Bishop, UnitKind::Knight, UnitKind::Rook];

        let mut board = Board { squares: [None; 64] };
        set_row(&mut board, Rank::Eight, create_row(Color::Black, pieces));
        set_row(&mut board, Rank::Seven, create_row(Color::Black, [UnitKind::Pawn; 8]));
        set_row(&mut board, Rank::Six, [None::<Unit>; 8]);
        set_row(&mut board, Rank::Five, [None::<Unit>; 8]);
        set_row(&mut board, Rank::Four, [None::<Unit>; 8]);
        set_row(&mut board, Rank::Three, [None::<Unit>; 8]);
        set_row(&mut board, Rank::Two, create_row(Color::White, [UnitKind::Pawn; 8]));
        set_row(&mut board, Rank::One, create_row(Color::White, pieces));

        Self {
            next_move: Color::White,
            white_can_king_side_castle: true,
            black_can_king_side_castle: true,
            white_can_queen_side_castle: true,
            black_can_queen_side_castle: true,
            en_passant_square: None,
            board: board,
            white_king_position: Square(File::E, Rank::One),
            black_king_position: Square(File::E, Rank::Eight)
        }
    }
}

pub fn get_other_color(color: Color) -> Color {
    match color {
        Color::White => Color::Black,
        Color::Black => Color::White
    }
}