extern crate chess;
use chess::{BitBoard, Board, Color, Piece};

pub fn create_bitboard(board: &Board, piece: Piece, color: Color) -> BitBoard {
    board.pieces(piece) & board.color_combined(color)
}

pub fn print_bitboard() {
    let board = Board::default();
    let white_pawns = create_bitboard(&board, Piece::Pawn, Color::White);
    let black_pawns = create_bitboard(&board, Piece::Pawn, Color::Black);

    println!("White Pawns: {:b}", white_pawns.0);
    println!("Black Pawns: {:b}", black_pawns.0);
}

pub fn get_bit(bitboard: BitBoard, square: usize) -> bool {
    (bitboard.0 & (1 << square)) != 0
}

pub fn set_bit(bitboard: &mut BitBoard, square: usize) {
    bitboard.0 |= 1 << square;
}

pub fn pop_bit(bitboard: &mut BitBoard, square: usize) {
    bitboard.0 &= !(1 << square);
}
