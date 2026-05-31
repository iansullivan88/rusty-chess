use crate::{Game, analysis::Algorithm::Random, moves::{Move, get_legal_moves}};
use rand::{Rng};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Algorithm {
    Random
}

pub fn choose_move(game: &mut Game, algorithm: Algorithm) -> Move {
    match algorithm {
        Random => choose_random_move(game)
    }
}

fn choose_random_move(game: &mut Game) -> Move {

    let legal_moves = get_legal_moves(game);

    let mut rng = rand::thread_rng();

    let random_index = rng.gen_range(0..legal_moves.len());

    legal_moves[random_index]
}