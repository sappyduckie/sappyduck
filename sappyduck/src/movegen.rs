extern crate chess;

use chess::{Board, MoveGen};
use std::str::FromStr;

#[derive(Clone)]
pub struct Position {
    pub board: Board,
    pub move_count: u32,
}

impl Position {
    pub fn from_fen(fen: &str) -> Self {
        let board = Board::from_str(fen).unwrap_or(Board::default());
        // Extract fullmove number from FEN if available
        let move_count = if let Some(parts) = fen.split_whitespace().nth(5) {
            parts.parse().unwrap_or(1) * 2 // Convert fullmove number to half moves
        } else {
            0
        };
        Position { board, move_count }
    }

    pub fn make_move(&mut self, mv: &str) -> bool {
        if let Ok(chess_move) = mv.parse() {
            self.board = self.board.make_move_new(chess_move);
            self.move_count += 1;
            true
        } else {
            false
        }
    }

    pub fn generate_legal_moves(&self) -> Vec<String> {
        let mut moves = Vec::new();
        for mv in MoveGen::new_legal(&self.board) {
            moves.push(mv.to_string());
        }
        moves
    }

    pub fn is_capture(&self, mv: &str) -> bool {
        if let Ok(chess_move) = mv.parse::<chess::ChessMove>() {
            self.board.piece_on(chess_move.get_dest()).is_some()
        } else {
            false
        }
    }
}
