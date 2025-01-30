extern crate chess;
use chess::*;
use lazy_static::lazy_static;

// Core constants
pub const FEN_START: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
pub const PIECE_TYPES: usize = 6;
pub const SQUARES: usize = 64;

// Bitboards for files
pub const FILE_A: BitBoard = BitBoard(0x0101010101010101);
pub const FILE_B: BitBoard = BitBoard(0x0202020202020202);
pub const FILE_G: BitBoard = BitBoard(0x4040404040404040);
pub const FILE_H: BitBoard = BitBoard(0x8080808080808080);

pub const RANK_2: BitBoard = BitBoard(0x000000000000FF00);
pub const RANK_7: BitBoard = BitBoard(0x00FF000000000000);

// Game phase constants
pub const OPENING_MOVES: u32 = 20;

// Piece values for different game phases
pub const QUEEN_VALUE_NORMAL: f64 = 9.5;
pub const QUEEN_VALUE_THRESHOLD_ADVANTAGE: f64 = 9.4;
pub const QUEEN_VALUE_SECOND_QUEEN: f64 = 8.7;

pub const FIRST_ROOK_OPENING: f64 = 5.63;
pub const FIRST_ROOK_MIDDLEGAME: f64 = 5.73;
pub const FIRST_ROOK_THRESHOLD: f64 = 5.73;
pub const FIRST_ROOK_ENDGAME: f64 = 6.13;

pub const SECOND_ROOK_OPENING: f64 = 5.63;
pub const SECOND_ROOK_MIDDLEGAME: f64 = 5.53;
pub const SECOND_ROOK_THRESHOLD: f64 = 5.93;
pub const SECOND_ROOK_ENDGAME: f64 = 6.03;

pub const BISHOP_VALUE: f64 = 3.33;
pub const BISHOP_PAIR_MIDDLEGAME: f64 = 0.3;
pub const BISHOP_PAIR_THRESHOLD: f64 = 0.4;
pub const BISHOP_PAIR_ENDGAME: f64 = 0.5;

pub const KNIGHT_VALUE_OPENING: f64 = 3.25;
pub const KNIGHT_VALUE_MIDDLEGAME: f64 = 3.2;
pub const KNIGHT_VALUE_THRESHOLD: f64 = 3.2;
pub const KNIGHT_VALUE_ENDGAME: f64 = 3.2;

pub const PAWN_VALUE_OPENING: f64 = 1.0;
pub const PAWN_VALUE_MIDDLEGAME: f64 = 0.8;
pub const PAWN_VALUE_THRESHOLD: f64 = 0.9;
pub const PAWN_VALUE_ENDGAME: f64 = 1.0;

pub const KING_VALUE: f64 = f64::INFINITY;

// Checkmate pattern bonuses
pub const BACK_RANK_MATE_BONUS: f64 = 5.0;
pub const SMOTHERED_MATE_BONUS: f64 = 4.0;

// Game phases
#[derive(PartialEq)]
pub enum GamePhase {
    Opening,
    Middlegame,
    Threshold,
    Endgame,
}

// Define pieces for easy reference
pub const KING: Piece = Piece::King;
pub const QUEEN: Piece = Piece::Queen;
pub const ROOK: Piece = Piece::Rook;
pub const BISHOP: Piece = Piece::Bishop;
pub const KNIGHT: Piece = Piece::Knight;
pub const PAWN: Piece = Piece::Pawn;

// Function implementations
pub fn is_original_position(board: &Board) -> bool {
    let initial_queens = (board.pieces(QUEEN)
        & (board.color_combined(Color::White) | board.color_combined(Color::Black)))
    .popcnt();
    initial_queens == 2
}

pub fn detect_game_phase(board: &Board, move_count: u32) -> GamePhase {
    if move_count <= OPENING_MOVES {
        return GamePhase::Opening;
    }

    let white_queens = (board.pieces(QUEEN) & board.color_combined(Color::White)).popcnt();
    let black_queens = (board.pieces(QUEEN) & board.color_combined(Color::Black)).popcnt();

    match (white_queens, black_queens) {
        (1, 1) if is_original_position(board) => GamePhase::Middlegame,
        (1, 1) => GamePhase::Middlegame,
        (1, 0) | (0, 1) => GamePhase::Threshold,
        (0, 0) => GamePhase::Endgame,
        _ => GamePhase::Middlegame,
    }
}

// Piece-square tables
pub const MG_PAWN_TABLE: [f64; 64] = [
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.98, 1.34, 0.61, 0.95, 0.68, 1.26, 0.34, -0.11, -0.06,
    0.07, 0.26, 0.31, 0.65, 0.56, 0.25, -0.20, -0.14, 0.13, 0.06, 0.21, 0.23, 0.12, 0.17, -0.23,
    -0.27, -0.02, -0.05, 0.12, 0.17, 0.06, 0.10, -0.25, -0.26, -0.04, -0.04, -0.10, 0.03, 0.03,
    0.33, -0.12, -0.35, -0.01, -0.20, -0.23, -0.15, 0.24, 0.38, -0.22, 0.0, 0.0, 0.0, 0.0, 0.0,
    0.0, 0.0, 0.0,
];

pub const EG_PAWN_TABLE: [f64; 64] = [
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.78, 1.73, 1.58, 1.34, 1.47, 1.32, 1.65, 1.87, 0.94,
    1.00, 0.85, 0.67, 0.56, 0.53, 0.82, 0.84, 0.32, 0.24, 0.13, 0.05, -0.02, 0.04, 0.17, 0.17,
    0.13, 0.09, -0.03, -0.07, -0.07, -0.08, 0.03, -0.01, 0.04, 0.07, -0.06, 0.01, 0.0, -0.05,
    -0.01, -0.08, 0.13, 0.08, 0.08, 0.10, 0.13, 0.0, 0.02, -0.07, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    0.0, 0.0,
];

pub const MG_KNIGHT_TABLE: [f64; 64] = [
    -1.67, -0.89, -0.34, -0.49, 0.61, -0.97, -0.15, -1.07, -0.73, -0.41, 0.72, 0.36, 0.23, 0.62,
    0.07, -0.17, -0.47, 0.60, 0.37, 0.65, 0.84, 1.29, 0.73, 0.44, -0.09, 0.17, 0.19, 0.53, 0.37,
    0.69, 0.18, 0.22, -0.13, 0.04, 0.16, 0.13, 0.28, 0.19, 0.21, -0.08, -0.23, -0.09, 0.12, 0.10,
    0.19, 0.17, 0.25, -0.16, -0.29, -0.53, -0.12, -0.03, -0.01, 0.18, -0.14, -0.19, -1.05, -0.21,
    -0.58, -0.33, -0.17, -0.28, -0.19, -0.23,
];

pub const EG_KNIGHT_TABLE: [f64; 64] = [
    -0.58, -0.38, -0.13, -0.28, -0.31, -0.27, -0.63, -0.99, -0.25, -0.08, -0.25, -0.02, -0.09,
    -0.25, -0.24, -0.52, -0.24, -0.20, 0.10, 0.09, -0.01, -0.09, -0.19, -0.41, -0.17, 0.03, 0.22,
    0.22, 0.22, 0.11, 0.08, -0.18, -0.18, -0.06, 0.16, 0.25, 0.16, 0.17, 0.04, -0.18, -0.23, -0.03,
    -0.01, 0.15, 0.10, -0.03, -0.20, -0.22, -0.42, -0.20, -0.10, -0.05, -0.02, -0.20, -0.23, -0.44,
    -0.29, -0.51, -0.23, -0.15, -0.22, -0.18, -0.50, -0.64,
];

pub const MG_BISHOP_TABLE: [f64; 64] = [
    -0.29, 0.04, -0.82, -0.37, -0.25, -0.42, 0.07, -0.08, -0.26, 0.16, -0.18, -0.13, 0.30, 0.59,
    0.18, -0.47, -0.16, 0.37, 0.43, 0.40, 0.35, 0.50, 0.37, -0.02, -0.04, 0.05, 0.19, 0.50, 0.37,
    0.37, 0.07, -0.02, -0.06, 0.13, 0.13, 0.26, 0.34, 0.12, 0.10, 0.04, 0.0, 0.15, 0.15, 0.15,
    0.14, 0.27, 0.18, 0.10, 0.04, 0.15, 0.16, 0.0, 0.07, 0.21, 0.33, 0.01, -0.33, -0.03, -0.14,
    -0.21, -0.13, -0.12, -0.39, -0.21,
];

pub const EG_BISHOP_TABLE: [f64; 64] = [
    -0.14, -0.21, -0.11, -0.08, -0.07, -0.09, -0.17, -0.24, -0.08, -0.04, 0.07, -0.12, -0.03,
    -0.13, -0.04, -0.14, 0.02, -0.08, 0.0, -0.01, -0.02, 0.06, 0.0, 0.04, -0.03, 0.09, 0.12, 0.09,
    0.14, 0.10, 0.03, 0.02, -0.06, 0.03, 0.13, 0.19, 0.07, 0.10, -0.03, -0.09, -0.12, -0.03, 0.08,
    0.10, 0.13, 0.03, -0.07, -0.15, -0.14, -0.18, -0.07, -0.01, 0.04, -0.09, -0.15, -0.27, -0.23,
    -0.09, -0.23, -0.05, -0.09, -0.16, -0.05, -0.17,
];

pub const MG_ROOK_TABLE: [f64; 64] = [
    0.32, 0.42, 0.32, 0.51, 0.63, 0.09, 0.31, 0.43, 0.27, 0.32, 0.58, 0.62, 0.80, 0.67, 0.26, 0.44,
    -0.05, 0.19, 0.26, 0.36, 0.17, 0.45, 0.61, 0.16, -0.24, -0.11, 0.07, 0.26, 0.24, 0.35, -0.08,
    -0.20, -0.36, -0.26, -0.12, -0.01, 0.09, -0.07, 0.06, -0.23, -0.45, -0.25, -0.16, -0.17, 0.03,
    0.00, -0.05, -0.33, -0.44, -0.16, -0.20, -0.09, -0.01, 0.11, -0.06, -0.71, -0.19, -0.13, 0.01,
    0.17, 0.16, 0.07, -0.37, -0.26,
];

pub const EG_ROOK_TABLE: [f64; 64] = [
    0.13, 0.10, 0.18, 0.15, 0.12, 0.12, 0.08, 0.05, 0.11, 0.13, 0.13, 0.11, -0.03, 0.03, 0.08,
    0.03, 0.07, 0.07, 0.07, 0.05, 0.04, -0.03, -0.05, -0.03, 0.04, 0.03, 0.13, 0.01, 0.02, 0.01,
    -0.01, 0.02, 0.03, 0.05, 0.08, 0.04, -0.05, -0.06, -0.08, -0.11, -0.04, 0.00, -0.05, -0.01,
    -0.07, -0.12, -0.08, -0.16, -0.06, -0.06, 0.00, 0.02, -0.09, -0.09, -0.11, -0.03, -0.09, 0.02,
    0.03, -0.01, -0.05, -0.13, 0.04, -0.20,
];

pub const MG_QUEEN_TABLE: [f64; 64] = [
    -0.28, 0.00, 0.29, 0.12, 0.59, 0.44, 0.43, 0.45, -0.24, -0.39, -0.05, 0.01, -0.16, 0.57, 0.28,
    0.54, -0.13, -0.17, 0.07, 0.08, 0.29, 0.56, 0.47, 0.57, -0.27, -0.27, -0.16, -0.16, -0.01,
    0.17, -0.02, 0.01, -0.09, -0.26, -0.09, -0.10, -0.02, -0.04, 0.03, -0.03, -0.14, 0.02, -0.11,
    -0.02, -0.05, 0.02, 0.14, 0.05, -0.35, -0.08, 0.11, 0.02, 0.08, 0.15, -0.03, 0.01, -0.01,
    -0.18, -0.09, 0.10, -0.15, -0.25, -0.31, -0.50,
];

pub const EG_QUEEN_TABLE: [f64; 64] = [
    -0.09, 0.22, 0.22, 0.27, 0.27, 0.19, 0.10, 0.20, -0.17, 0.20, 0.32, 0.41, 0.58, 0.25, 0.30,
    0.00, -0.20, 0.06, 0.09, 0.49, 0.47, 0.35, 0.19, 0.09, 0.03, 0.22, 0.24, 0.45, 0.57, 0.40,
    0.57, 0.36, -0.18, 0.28, 0.19, 0.47, 0.31, 0.34, 0.39, 0.23, -0.16, -0.27, 0.15, 0.06, 0.09,
    0.17, 0.10, 0.05, -0.22, -0.23, -0.30, -0.16, -0.16, -0.23, -0.36, -0.32, -0.33, -0.28, -0.22,
    -0.43, -0.05, -0.32, -0.20, -0.41,
];

pub const MG_KING_TABLE: [f64; 64] = [
    -0.65, 0.23, 0.16, -0.15, -0.56, -0.34, 0.02, 0.13, 0.29, -0.01, -0.20, -0.07, -0.08, -0.04,
    -0.38, -0.29, -0.09, 0.24, 0.02, -0.16, -0.20, 0.06, 0.22, -0.22, -0.17, -0.20, -0.12, -0.27,
    -0.30, -0.25, -0.14, -0.36, -0.49, -0.01, -0.27, -0.39, -0.46, -0.44, -0.33, -0.51, -0.14,
    -0.14, -0.22, -0.46, -0.44, -0.30, -0.15, -0.27, 0.01, 0.07, -0.08, -0.64, -0.43, -0.16, 0.09,
    0.08, -0.15, 0.36, 0.12, -0.54, 0.08, -0.28, 0.24, 0.14,
];

pub const EG_KING_TABLE: [f64; 64] = [
    -0.74, -0.35, -0.18, -0.18, -0.11, 0.15, 0.04, -0.17, -0.12, 0.17, 0.14, 0.17, 0.17, 0.38,
    0.23, 0.11, 0.10, 0.17, 0.23, 0.15, 0.20, 0.45, 0.44, 0.13, -0.08, 0.22, 0.24, 0.27, 0.26,
    0.33, 0.26, 0.03, -0.18, -0.04, 0.21, 0.24, 0.27, 0.23, 0.09, -0.11, -0.19, -0.03, 0.11, 0.21,
    0.23, 0.16, 0.07, -0.09, -0.27, -0.11, 0.04, 0.13, 0.14, 0.04, -0.05, -0.17, -0.53, -0.34,
    -0.21, -0.11, -0.28, -0.14, -0.24, -0.43,
];

// Piece value getters
pub fn get_pawn_value(phase: &GamePhase) -> f64 {
    match phase {
        GamePhase::Opening => PAWN_VALUE_OPENING,
        GamePhase::Middlegame => PAWN_VALUE_MIDDLEGAME,
        GamePhase::Threshold => PAWN_VALUE_THRESHOLD,
        GamePhase::Endgame => PAWN_VALUE_ENDGAME,
    }
}

pub fn get_knight_value(phase: &GamePhase) -> f64 {
    match phase {
        GamePhase::Opening => KNIGHT_VALUE_OPENING,
        GamePhase::Middlegame => KNIGHT_VALUE_MIDDLEGAME,
        GamePhase::Threshold => KNIGHT_VALUE_THRESHOLD,
        GamePhase::Endgame => KNIGHT_VALUE_ENDGAME,
    }
}

pub fn get_bishop_pair_bonus(phase: &GamePhase) -> f64 {
    match phase {
        GamePhase::Opening => 0.0,
        GamePhase::Middlegame => BISHOP_PAIR_MIDDLEGAME,
        GamePhase::Threshold => BISHOP_PAIR_THRESHOLD,
        GamePhase::Endgame => BISHOP_PAIR_ENDGAME,
    }
}

pub fn get_rook_value(phase: &GamePhase, is_first_rook: bool) -> f64 {
    match (phase, is_first_rook) {
        // First rook values
        (GamePhase::Opening, true) => FIRST_ROOK_OPENING,
        (GamePhase::Middlegame, true) => FIRST_ROOK_MIDDLEGAME,
        (GamePhase::Threshold, true) => FIRST_ROOK_THRESHOLD,
        (GamePhase::Endgame, true) => FIRST_ROOK_ENDGAME,
        // Second rook values
        (GamePhase::Opening, false) => SECOND_ROOK_OPENING,
        (GamePhase::Middlegame, false) => SECOND_ROOK_MIDDLEGAME,
        (GamePhase::Threshold, false) => SECOND_ROOK_THRESHOLD,
        (GamePhase::Endgame, false) => SECOND_ROOK_ENDGAME,
    }
}

// Helper function to flip table indices for black's perspective
pub fn flip_vertical(sq: usize) -> usize {
    sq ^ 56 // Exclusive OR with 56 (7 * 8) flips between ranks
}

// Function to get piece square value based on color and game phase
pub fn get_piece_square_value(piece: Piece, square: usize, color: Color, phase: &GamePhase) -> f64 {
    let sq = if color == Color::Black {
        flip_vertical(square)
    } else {
        square
    };

    match (piece, phase) {
        (Piece::Pawn, GamePhase::Middlegame) => MG_PAWN_TABLE[sq],
        (Piece::Pawn, GamePhase::Endgame) => EG_PAWN_TABLE[sq],
        (Piece::Knight, GamePhase::Middlegame) => MG_KNIGHT_TABLE[sq],
        (Piece::Knight, GamePhase::Endgame) => EG_KNIGHT_TABLE[sq],
        (Piece::Bishop, GamePhase::Middlegame) => MG_BISHOP_TABLE[sq],
        (Piece::Bishop, GamePhase::Endgame) => EG_BISHOP_TABLE[sq],
        (Piece::Rook, GamePhase::Middlegame) => MG_ROOK_TABLE[sq],
        (Piece::Rook, GamePhase::Endgame) => EG_ROOK_TABLE[sq],
        (Piece::Queen, GamePhase::Middlegame) => MG_QUEEN_TABLE[sq],
        (Piece::Queen, GamePhase::Endgame) => EG_QUEEN_TABLE[sq],
        (Piece::King, GamePhase::Middlegame) => MG_KING_TABLE[sq],
        (Piece::King, GamePhase::Endgame) => EG_KING_TABLE[sq],
        _ => 0.0,
    }
}

// Bitboard definitions using lazy_static
lazy_static! {
    // Precomputed bitboards for piece attacks
    pub static ref PAWN_ATTACKS: [[BitBoard; SQUARES]; 2] = {
        let mut attacks = [[BitBoard(0); SQUARES]; 2];
        for sq in 0..SQUARES {
            let bb = 1u64 << sq;
            // White pawn attacks
            attacks[Color::White as usize][sq] = BitBoard(
                ((bb << 7) & !FILE_A.0) | ((bb << 9) & !FILE_H.0)
            );
            // Black pawn attacks
            attacks[Color::Black as usize][sq] = BitBoard(
                ((bb >> 7) & !FILE_H.0) | ((bb >> 9) & !FILE_A.0)
            );
        }
        attacks
    };

    pub static ref KNIGHT_ATTACKS: [BitBoard; SQUARES] = {
        let mut attacks = [BitBoard(0); SQUARES];
        for sq in 0..SQUARES {
            let bb = 1u64 << sq;
            attacks[sq] = BitBoard(
                ((bb << 17) & !FILE_A.0) |
                ((bb << 15) & !FILE_H.0) |
                ((bb << 10) & !(FILE_A.0 | FILE_B.0)) |
                ((bb << 6) & !(FILE_G.0 | FILE_H.0)) |
                ((bb >> 6) & !(FILE_A.0 | FILE_B.0)) |
                ((bb >> 10) & !(FILE_G.0 | FILE_H.0)) |
                ((bb >> 15) & !FILE_A.0) |
                ((bb >> 17) & !FILE_H.0)
            );
        }
        attacks
    };

    pub static ref KING_ATTACKS: [BitBoard; SQUARES] = {
        let mut attacks = [BitBoard(0); SQUARES];
        for sq in 0..SQUARES {
            let bb = 1u64 << sq;
            attacks[sq] = BitBoard(
                ((bb << 8) | (bb >> 8)) |
                ((bb << 1) & !FILE_A.0) |
                ((bb >> 1) & !FILE_H.0) |
                ((bb << 7) & !FILE_H.0) |
                ((bb << 9) & !FILE_A.0) |
                ((bb >> 7) & !FILE_A.0) |
                ((bb >> 9) & !FILE_H.0)
            );
        }
        attacks
    };

    pub static ref BISHOP_ATTACKS: [BitBoard; SQUARES] = {
        let mut attacks = [BitBoard(0); SQUARES];
        for sq in 0..SQUARES {
            let rank = sq / 8;
            let file = sq % 8;
            let mut bb = 0u64;

            // Generate diagonal attacks in all four directions
            for i in 1..8 {
                if rank + i < 8 && file + i < 8 { bb |= 1u64 << (sq + i * 9); }
                if rank + i < 8 && file >= i { bb |= 1u64 << (sq + i * 7); }
                if rank >= i && file + i < 8 { bb |= 1u64 << (sq - i * 7); }
                if rank >= i && file >= i { bb |= 1u64 << (sq - i * 9); }
            }
            attacks[sq] = BitBoard(bb);
        }
        attacks
    };

    pub static ref ROOK_ATTACKS: [BitBoard; SQUARES] = {
        let mut attacks = [BitBoard(0); SQUARES];
        for sq in 0..SQUARES {
            let rank = sq / 8;
            let file = sq % 8;
            let mut bb = 0u64;

            // Generate horizontal and vertical attacks
            for i in 1..8 {
                if file + i < 8 { bb |= 1u64 << (sq + i); }
                if file >= i { bb |= 1u64 << (sq - i); }
                if rank + i < 8 { bb |= 1u64 << (sq + i * 8); }
                if rank >= i { bb |= 1u64 << (sq - i * 8); }
            }
            attacks[sq] = BitBoard(bb);
        }
        attacks
    };

    pub static ref QUEEN_ATTACKS: [BitBoard; SQUARES] = {
        let mut attacks = [BitBoard(0); SQUARES];
        for sq in 0..SQUARES {
            attacks[sq] = BitBoard(BISHOP_ATTACKS[sq].0 | ROOK_ATTACKS[sq].0);
        }
        attacks
    };

    pub static ref KING_SAFETY_MASK: [BitBoard; SQUARES] = {
        let mut masks = [BitBoard(0); SQUARES];
        for sq in 0..SQUARES {
            let king_bb = 1u64 << sq;
            masks[sq] = BitBoard(
                ((king_bb << 8) | (king_bb >> 8) |
                (king_bb << 1 & !FILE_A.0) |
                (king_bb >> 1 & !FILE_H.0) |
                (king_bb << 7 & !FILE_H.0) |
                (king_bb << 9 & !FILE_A.0) |
                (king_bb >> 7 & !FILE_A.0) |
                (king_bb >> 9 & !FILE_H.0))
            );
        }
        masks
    };
}
