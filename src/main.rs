use chess::{ALL_FILES, ALL_RANKS, Board, Color, Game, Square, Unit, UnitKind, moves::commands::parse_move};
use std::io::{self, Write, BufWriter};

fn main() {
    run_game().expect("Unexpected error");
}

fn run_game() -> io::Result<()>  {
    let game = Game::new();
    let mut input = String::new();

    loop {
        print_board(&game.board)?;
        println!("Enter move for {}", color_string(game.next_move));

        input.clear();
        io::stdin().read_line(&mut input)?;
        
        let chess_move = match parse_move(&input, &game) {
            Err(e) => {
                println!("Invalid move: {}", e);
                continue;
            }
            Ok(chess_moves) => chess_moves
        };

        println!("Move: {:?}", chess_move);
    }
}

fn print_board(board: &Board) -> io::Result<()> {

    let light_wood = b"\x1B[48;2;213;176;124m";
    let dark_wood  = b"\x1B[48;2;125;88;57m";
    let reset = b"\x1B[0m";

    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());
    
    for (rank_index, &rank) in ALL_RANKS.iter().rev().enumerate() {
        for (file_index, &file) in ALL_FILES.iter().enumerate() {
            if (rank_index + file_index) % 2 == 0 {
                writer.write_all(light_wood)?;
            } else {
                writer.write_all(dark_wood)?;
            }

            print_unit(&mut writer, board[Square(file, rank)])?;

            writer.write_all(reset)?;
        }

        write!(writer, "\n")?;
    }

    writer.write_all(reset)?;

    io::Result::Ok(())
}

fn print_unit(writer: &mut BufWriter<io::StdoutLock<'_>>, maybe_unit: Option<Unit>) -> io::Result<()> {
    
    // print color if required
    if let Some(unit) = maybe_unit {
        let color: &[u8] = match unit.color {
            Color::White => b"\x1b[38;5;255m",
            Color::Black => b"\x1b[38;5;0m"
        };

        writer.write_all(color)?
    }

    let character = match maybe_unit {
        None => " ",
        Some(Unit { kind: UnitKind::Bishop, color: Color::White}) => "♗",
        Some(Unit { kind: UnitKind::King, color: Color::White}) => "♔",
        Some(Unit { kind: UnitKind::Knight, color: Color::White}) => "♘",
        Some(Unit { kind: UnitKind::Pawn, color: Color::White}) => "♙",
        Some(Unit { kind: UnitKind::Queen, color: Color::White}) => "♕",
        Some(Unit { kind: UnitKind::Rook, color: Color::White}) => "♖",
        Some(Unit { kind: UnitKind::Bishop, color: Color::Black}) => "♝",
        Some(Unit { kind: UnitKind::King, color: Color::Black}) => "♚",
        Some(Unit { kind: UnitKind::Knight, color: Color::Black}) => "♞",
        Some(Unit { kind: UnitKind::Pawn, color: Color::Black}) => "♟",
        Some(Unit { kind: UnitKind::Queen, color: Color::Black}) => "♛",
        Some(Unit { kind: UnitKind::Rook, color: Color::Black}) => "♜"
    };

    write!(writer, "{character} ")
}

fn color_string (color: Color) -> String {
    let color_slice = match color {
        Color::Black => "black",
        Color::White => "white"
    };
    String::from(color_slice)
}