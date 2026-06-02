use chess::{ALL_FILES, ALL_RANKS, Board, Color, Game, Square, Unit, UnitKind, analysis::{Algorithm, choose_move}, get_other_color, moves::{Move, apply_move_to_game, commands::parse_move, get_legal_moves, is_king_in_check}};
use std::io::{self, BufWriter, StdoutLock, Write};

fn main() {
    run_game().expect("Unexpected error");
}

fn run_game() -> io::Result<()>  {
    let mut game = Game::new();
    let mut input = String::new();
    let algorithm = Algorithm::Random;

    loop {
        print_board(&game.board)?;

        let legal_moves = get_legal_moves(&mut game);

        if legal_moves.len() == 0 {
            if is_king_in_check(&game, game.next_move) {
                println!("{} won!", get_other_color(game.next_move));
            } else {
                println!("Stalemate");
            }
            
            break;
        }

        let r#move = match game.next_move {
            Color::White => {
                println!("Enter move for {}", color_string(game.next_move));

                input.clear();
                io::stdin().read_line(&mut input)?;
                
                match parse_move(&input, &mut game) {
                    Err(e) => {
                        println!("Invalid move: {}", e);
                        continue;
                    }
                    Ok(r#move) => r#move
                }
            }
            Color::Black => {
                let stdout = io::stdout();
                let mut writer = BufWriter::new(stdout.lock());
                let chosen_move = choose_move(&mut game, algorithm);
                print_move(&mut writer, &game.board, chosen_move)?;
                write!(writer, "\n")?;
                write!(writer, "\n")?;
                chosen_move

            }
        };

        apply_move_to_game(&mut game, r#move);

        
    }

    Ok(())
}

fn print_board(board: &Board) -> io::Result<()> {

    fn write_files(writer: &mut BufWriter<StdoutLock<'_>>) -> io::Result<()> {
        write!(writer, "  ")?;
        for file in ALL_FILES {
            write!(writer, "{} " , file)?;
        }
        write!(writer, "\n")?;
        Ok(())
    }

    let light_wood = b"\x1B[48;2;213;176;124m";
    let dark_wood  = b"\x1B[48;2;125;88;57m";
    let reset = b"\x1B[0m";

    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());
    
    write_files(&mut writer)?;
    for (rank_index, &rank) in ALL_RANKS.iter().rev().enumerate() {
        write!(writer, "{} ", rank)?;

        for (file_index, &file) in ALL_FILES.iter().enumerate() {
            if (rank_index + file_index) % 2 == 0 {
                writer.write_all(light_wood)?;
            } else {
                writer.write_all(dark_wood)?;
            }

            print_unit(&mut writer, board[Square(file, rank)])?;

            writer.write_all(reset)?;
        }

        write!(writer, " {}\n", rank)?;
    }

    write_files(&mut writer)?;
    
    write!(writer, "\n")?;

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

// not a perfect implementation of notation but close enough
fn print_move(writer: &mut BufWriter<io::StdoutLock<'_>>, board: &Board, r#move: Move) -> io::Result<()> {
    fn format_unit_kind(kind: UnitKind) -> String {
        let slice = match kind {
            UnitKind::Bishop => "B",
            UnitKind::King => "K",
            UnitKind::Knight => "N",
            UnitKind::Pawn => "",
            UnitKind::Queen => "Q",
            UnitKind::Rook => "R"
        };

        String::from(slice)
    }
    
    fn print_move(f: &mut BufWriter<io::StdoutLock<'_>>, from: Square, to: Square, kind: UnitKind, promote_to: Option<UnitKind>, is_capture: bool) -> io::Result<()> {
        
        let promotion_text = promote_to.map_or(String::new(), |p| format!("={}", format_unit_kind(p)));

        write!(f, "{}{}{}{}{}{}{}",
            format_unit_kind(kind),
            from.0.to_string().to_lowercase(),
            from.1,
            if is_capture { "x" } else { "" },
            to.0.to_string().to_lowercase(),
            to.1,
            promotion_text)
    }

    match r#move {
        Move::KingSideCastle => write!(writer, "o-o"),
        Move::QueenSideCastle => write!(writer, "o-o-o"),
        Move::Normal { from, to, captured_unit, .. } if let Some(Unit { kind, .. }) = board[from] => print_move(writer, from, to, kind, None, captured_unit.is_some()),
        Move::Promotion { from, to,  captured_unit, promote_to } => print_move(writer, from, to, UnitKind::Pawn, Some(promote_to), captured_unit.is_some()),
        Move::EnPassant { from, to } => print_move(writer, from, to, UnitKind::Pawn, None, true),
        _ => write!(writer, "Cannot print move"),
    }
}