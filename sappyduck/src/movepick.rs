use crate::defs::*;
use crate::movegen::Position;
use crate::uci::should_stop;
use chess::{BitBoard, Board, ChessMove, Color, File, Piece, Rank, Square};
use std::time::{Duration, Instant};

pub struct SearchParams {
    pub depth: i32,
    pub start_time: Instant,
    pub max_time: Duration,
    pub nodes: u64,
}

impl Default for SearchParams {
    fn default() -> Self {
        SearchParams {
            depth: 0,
            start_time: Instant::now(),
            max_time: Duration::from_secs(5),
            nodes: 0,
        }
    }
}

// Modify pick_move to use iterative deepening
pub fn pick_move(position: &mut Position) -> Option<String> {
    let mut params = SearchParams::default();
    let mut best_move = None;
    let mut best_score = f64::NEG_INFINITY;
    let max_depth = 1; // Changed from 20 to 1
    let window_size = 0.5; // Aspiration window size in pawns

    // Initial info to GUI
    println!(
        "info string starting search at position with {} moves",
        position.move_count
    );

    // Get all legal moves at the start
    let legal_moves = position.generate_legal_moves();
    if legal_moves.is_empty() {
        return None;
    }

    // Always have a move ready
    best_move = legal_moves.first().cloned();

    for depth in 1..=max_depth {
        params.depth = depth;
        params.start_time = Instant::now();

        // Use aspiration windows for deeper searches
        let mut alpha = if depth >= 4 {
            best_score - window_size
        } else {
            f64::NEG_INFINITY
        };
        let mut beta = if depth >= 4 {
            best_score + window_size
        } else {
            f64::INFINITY
        };

        let mut research_needed = true;
        while research_needed {
            let (score, mv) = alpha_beta_search(
                position,
                depth,
                alpha,
                beta,
                position.board.side_to_move() == Color::White,
                &mut params,
            );

            if score <= alpha {
                alpha = f64::NEG_INFINITY;
                continue;
            }
            if score >= beta {
                beta = f64::INFINITY;
                continue;
            }

            research_needed = false;

            if mv.is_some() {
                // Only update if score is better or it's the first move
                if score > best_score || best_move.is_none() {
                    best_move = mv;
                    best_score = score;
                }
            }

            // Always print info for GUI
            println!(
                "info depth {} score cp {} nodes {} time {} pv {}",
                depth,
                (best_score * 100.0) as i32,
                params.nodes,
                params.start_time.elapsed().as_millis(),
                best_move.as_ref().unwrap_or(&"(none)".to_string())
            );
        }

        if params.start_time.elapsed() >= params.max_time || should_stop() {
            break;
        }
    }

    best_move
}

// Add move ordering function
fn order_moves(moves: &mut Vec<String>, position: &Position) {
    moves.sort_by_cached_key(|mv| {
        let mut score = 0;
        if let Ok(chess_move) = mv.parse::<ChessMove>() {
            // Prioritize captures based on MVV-LVA (Most Valuable Victim - Least Valuable Attacker)
            if let Some(captured_piece) = position.board.piece_on(chess_move.get_dest()) {
                let attacker = position.board.piece_on(chess_move.get_source()).unwrap();
                score += 10 * get_piece_value(captured_piece) - get_piece_value(attacker);
            }

            // Center control bonus
            let dest = chess_move.get_dest().to_index();
            if (27..=36).contains(&dest) {
                score += 50;
            }

            // Development bonus in opening
            if position.move_count < 10 {
                if is_development_move(&position.board, chess_move) {
                    score += 30;
                }
            }

            // King safety consideration
            if is_king_safety_move(&position.board, chess_move) {
                score += 40;
            }

            // Penalty for moving pieces multiple times in opening
            if position.move_count < 10 && is_repeat_move(&position.board, chess_move) {
                score -= 20;
            }
        }
        -score // Negative for descending order
    });
}

fn get_piece_value(piece: Piece) -> i32 {
    match piece {
        Piece::Pawn => 100,
        Piece::Knight => 320,
        Piece::Bishop => 330,
        Piece::Rook => 500,
        Piece::Queen => 900,
        Piece::King => 20000,
    }
}

fn is_development_move(board: &Board, mv: ChessMove) -> bool {
    let piece = board.piece_on(mv.get_source()).unwrap();
    let from_rank = mv.get_source().get_rank().to_index();
    let is_home_rank = match board.side_to_move() {
        Color::White => from_rank == 1,
        Color::Black => from_rank == 6,
    };

    matches!(piece, Piece::Knight | Piece::Bishop) && is_home_rank
}

fn is_king_safety_move(board: &Board, mv: ChessMove) -> bool {
    let piece = board.piece_on(mv.get_source()).unwrap();
    piece == Piece::King && board.checkers().0 != 0
}

fn is_repeat_move(board: &Board, mv: ChessMove) -> bool {
    let piece = board.piece_on(mv.get_source()).unwrap();
    let from_rank = mv.get_source().get_rank().to_index();
    match board.side_to_move() {
        Color::White => from_rank > 1 && piece != Piece::King && piece != Piece::Queen,
        Color::Black => from_rank < 6 && piece != Piece::King && piece != Piece::Queen,
    }
}

pub fn alpha_beta_search(
    position: &Position,
    depth: i32,
    mut alpha: f64,
    mut beta: f64,
    is_maximizing: bool,
    params: &mut SearchParams,
) -> (f64, Option<String>) {
    if depth == 0 || should_stop() {
        return (evaluate_board(&position.board, position.move_count), None);
    }

    let mut moves = position.generate_legal_moves();
    order_moves(&mut moves, position);
    if moves.is_empty() {
        if position.board.checkers().0 != 0 {
            // If in check with no moves, it's checkmate
            return (-10000.0 + depth as f64, None);
        }
        // If not in check with no moves, it's stalemate
        return (0.0, None);
    }

    let mut best_move = None;
    let mut best_value = if is_maximizing {
        f64::NEG_INFINITY
    } else {
        f64::INFINITY
    };

    for mv in moves {
        let mut new_position = position.clone();
        let mv: String = mv;
        if new_position.make_move(&mv) {
            let (eval, _) = alpha_beta_search(
                &new_position,
                depth - 1,
                alpha,
                beta,
                !is_maximizing,
                params,
            );

            if is_maximizing && eval > best_value {
                best_value = eval;
                best_move = Some(mv);
                alpha = alpha.max(eval);
            } else if !is_maximizing && eval < best_value {
                best_value = eval;
                best_move = Some(mv);
                beta = beta.min(eval);
            }

            if beta <= alpha {
                break;
            }
        }
    }

    (best_value, best_move)
}

struct AttackInfo {
    attackers: Vec<(Piece, usize)>, // (piece type, square)
    defenders: Vec<(Piece, usize)>,
    target_value: f64,
    smallest_attacker: f64,
    smallest_defender: f64,
}

struct RookInfo {
    is_first_rook: bool,
    is_open_file: bool,
    is_semi_open: bool,
    controls_seventh: bool,
}

fn analyze_rook_position(board: &Board, square: usize, color: Color) -> RookInfo {
    let rook_bb = BitBoard(1 << square);
    let own_pawns = board.pieces(PAWN) & board.color_combined(color);
    let enemy_pawns = board.pieces(PAWN) & board.color_combined(!color);
    let file_mask = match square % 8 {
        0 => FILE_A,
        1 => FILE_B,
        6 => FILE_G,
        7 => FILE_H,
        _ => BitBoard((0x0101010101010101u64) << (square % 8)),
    };

    let seventh_rank = if color == Color::White {
        RANK_7
    } else {
        RANK_2
    };

    RookInfo {
        is_first_rook: true, // Will be adjusted in evaluate_material
        is_open_file: (file_mask & (own_pawns | enemy_pawns)).0 == 0,
        is_semi_open: (file_mask & own_pawns).0 == 0,
        controls_seventh: (rook_bb & seventh_rank).0 != 0,
    }
}

fn get_rook_position_bonus(info: &RookInfo) -> f64 {
    let mut bonus = 0.0;

    if info.is_open_file {
        bonus += 0.3;
    } else if info.is_semi_open {
        bonus += 0.15;
    }

    if info.controls_seventh {
        bonus += 0.25;
    }

    bonus
}

fn get_piece_base_value(piece: Piece, phase: &GamePhase) -> f64 {
    match (piece, phase) {
        // Pawn values
        (Piece::Pawn, GamePhase::Opening) => PAWN_VALUE_OPENING,
        (Piece::Pawn, GamePhase::Middlegame) => PAWN_VALUE_MIDDLEGAME,
        (Piece::Pawn, GamePhase::Threshold) => PAWN_VALUE_THRESHOLD,
        (Piece::Pawn, GamePhase::Endgame) => PAWN_VALUE_ENDGAME,

        // Knight values
        (Piece::Knight, GamePhase::Opening) => KNIGHT_VALUE_OPENING,
        (Piece::Knight, GamePhase::Middlegame) => KNIGHT_VALUE_MIDDLEGAME,
        (Piece::Knight, GamePhase::Threshold) => KNIGHT_VALUE_THRESHOLD,
        (Piece::Knight, GamePhase::Endgame) => KNIGHT_VALUE_ENDGAME,

        // Bishop values - constant across phases except for pair bonus
        (Piece::Bishop, _) => BISHOP_VALUE,

        // Rook values (using first rook values as default)
        (Piece::Rook, GamePhase::Opening) => FIRST_ROOK_OPENING,
        (Piece::Rook, GamePhase::Middlegame) => FIRST_ROOK_MIDDLEGAME,
        (Piece::Rook, GamePhase::Threshold) => FIRST_ROOK_THRESHOLD,
        (Piece::Rook, GamePhase::Endgame) => FIRST_ROOK_ENDGAME,

        // Queen values
        (Piece::Queen, GamePhase::Opening) => QUEEN_VALUE_NORMAL,
        (Piece::Queen, GamePhase::Middlegame) => QUEEN_VALUE_NORMAL,
        (Piece::Queen, GamePhase::Threshold) => QUEEN_VALUE_THRESHOLD_ADVANTAGE,
        (Piece::Queen, GamePhase::Endgame) => QUEEN_VALUE_NORMAL,

        // King value is always infinity
        (Piece::King, _) => KING_VALUE,
    }
}

fn evaluate_square_control(board: &Board, square: usize, color: Color) -> AttackInfo {
    let mut info = AttackInfo {
        attackers: Vec::new(),
        defenders: Vec::new(),
        target_value: get_piece_value_on_square(board, square),
        smallest_attacker: f64::INFINITY,
        smallest_defender: f64::INFINITY,
    };

    let phase = detect_game_phase(board, 0); // Get current game phase

    // Check attacks for each piece type
    for piece in &[PAWN, KNIGHT, BISHOP, ROOK, QUEEN] {
        let piece_bb = board.pieces(*piece) & board.color_combined(color);
        let attacks = match piece {
            &PAWN => PAWN_ATTACKS[color as usize][square],
            &KNIGHT => KNIGHT_ATTACKS[square],
            &BISHOP => BISHOP_ATTACKS[square],
            &ROOK => ROOK_ATTACKS[square],
            &QUEEN => QUEEN_ATTACKS[square],
            _ => BitBoard(0),
        };

        if (piece_bb & attacks).0 != 0 {
            let piece_value = get_piece_base_value(*piece, &phase);
            info.attackers.push((*piece, square));
            info.smallest_attacker = info.smallest_attacker.min(piece_value);
        }
    }

    info
}

struct SEEResult {
    gain: f64,
    exchange_sequence: Vec<(Piece, usize)>,
}

fn static_exchange_evaluation(board: &Board, square: usize, attacking_color: Color) -> SEEResult {
    let mut result = SEEResult {
        gain: 0.0,
        exchange_sequence: Vec::new(),
    };

    let target_value = get_piece_value_on_square(board, square);
    let mut current_value = target_value;
    let mut attacker_value = f64::INFINITY;
    let phase = detect_game_phase(board, 0); // Add this line to get the game phase

    // Find smallest attacker
    for piece in &[PAWN, KNIGHT, BISHOP, ROOK, QUEEN] {
        let attackers = board.pieces(*piece) & board.color_combined(attacking_color);
        let attack_pattern = match piece {
            &PAWN => PAWN_ATTACKS[attacking_color as usize][square],
            &KNIGHT => KNIGHT_ATTACKS[square],
            &BISHOP => BISHOP_ATTACKS[square],
            &ROOK => ROOK_ATTACKS[square],
            &QUEEN => QUEEN_ATTACKS[square],
            _ => BitBoard(0),
        };

        if (attackers & attack_pattern).0 != 0 {
            attacker_value = get_piece_base_value(*piece, &phase);
            result.exchange_sequence.push((*piece, square));
            break;
        }
    }

    result.gain = if attacker_value < f64::INFINITY {
        target_value - attacker_value
    } else {
        0.0
    };

    result
}

fn evaluate_attacks(board: &Board, square: usize, color: Color) -> f64 {
    let attack_info = evaluate_square_control(board, square, color);
    let defense_info = evaluate_square_control(board, square, !color);

    if attack_info.attackers.is_empty() {
        return 0.0;
    }

    // Base attack value
    let mut attack_value = attack_info.target_value - attack_info.smallest_attacker;

    // Multiple attacker bonus
    let attacker_bonus = match attack_info.attackers.len() {
        2 => 0.3,
        3 => 0.5,
        4.. => 0.7,
        _ => 0.0,
    };

    // Defense penalty
    let defense_penalty = if !defense_info.defenders.is_empty() {
        -0.1 * defense_info.defenders.len() as f64
    } else {
        0.0
    };

    // Hanging piece bonus (undefended target)
    let hanging_bonus = if defense_info.defenders.is_empty() {
        0.3
    } else {
        0.0
    };

    let mut total_value = attack_value + attacker_bonus + defense_penalty + hanging_bonus;

    // Add SEE evaluation for captures
    let see_result = static_exchange_evaluation(board, square, color);
    total_value += see_result.gain;

    // Add bonus for favorable exchanges
    if !see_result.exchange_sequence.is_empty() && see_result.gain > 0.0 {
        total_value += 0.2; // Bonus for winning exchange
    }

    total_value
}

fn get_piece_value_on_square(board: &Board, square: usize) -> f64 {
    let square_bb = BitBoard(1 << square);
    let phase = detect_game_phase(board, 0);

    for piece in [QUEEN, ROOK, BISHOP, KNIGHT, PAWN].iter() {
        if (board.pieces(*piece) & square_bb).0 != 0 {
            return get_piece_base_value(*piece, &phase);
        }
    }
    0.0
}

fn detect_checkmate_patterns(board: &Board, color: Color) -> f64 {
    let mut pattern_value = 0.0;

    // Find king's square using BitBoard's built-in methods
    let king_bb = board.pieces(KING) & board.color_combined(!color);
    let king_square = if king_bb.0 != 0 {
        // Find the least significant bit position (first 1 in the bitboard)
        let square_index = king_bb.0.trailing_zeros() as usize;
        // Create Square using the correct method
        Square::make_square(
            Rank::from_index(square_index / 8),
            File::from_index(square_index % 8),
        )
    } else {
        return 0.0; // No king found (shouldn't happen in a valid position)
    };

    // Back rank mate pattern
    if detect_back_rank_mate(board, king_square, !color) {
        pattern_value += BACK_RANK_MATE_BONUS;
    }

    // Smothered mate pattern
    if detect_smothered_mate(board, king_square, !color) {
        pattern_value += SMOTHERED_MATE_BONUS;
    }

    pattern_value
}

fn detect_back_rank_mate(board: &Board, king_sq: Square, king_color: Color) -> bool {
    let rank = king_sq.get_rank().to_index();
    let back_rank = if king_color == Color::White { 0 } else { 7 };

    if rank != back_rank {
        return false;
    }

    // Check if king is blocked by own pieces
    let king_zone = KING_SAFETY_MASK[king_sq.to_index()];
    let friendly_pieces = board.color_combined(king_color);
    let escape_squares = king_zone & !friendly_pieces;

    // Check if enemy rook or queen controls the back rank
    let enemy_pieces = board.color_combined(!king_color);
    let enemy_rooks = board.pieces(ROOK) & enemy_pieces;
    let enemy_queens = board.pieces(QUEEN) & enemy_pieces;

    escape_squares.0 == 0 && (enemy_rooks.0 != 0 || enemy_queens.0 != 0)
}

fn detect_smothered_mate(board: &Board, king_sq: Square, king_color: Color) -> bool {
    let king_zone = KING_SAFETY_MASK[king_sq.to_index()];
    let friendly_pieces = board.color_combined(king_color);

    // King must be surrounded by friendly pieces
    if (king_zone & !friendly_pieces).0 != 0 {
        return false;
    }

    // Check for enemy knight giving check
    let enemy_knights = board.pieces(KNIGHT) & board.color_combined(!king_color);
    KNIGHT_ATTACKS[king_sq.to_index()].0 & enemy_knights.0 != 0
}

pub fn evaluate_board(board: &Board, move_count: u32) -> f64 {
    let mut white_value = 0.0;
    let mut black_value = 0.0;
    let phase = detect_game_phase(board, move_count);

    // Add positional values for each piece
    for square in 0..64 {
        let sq_bb = BitBoard(1 << square);

        // For each piece type on this square
        for &piece in &[KING, QUEEN, ROOK, BISHOP, KNIGHT, PAWN] {
            let piece_bb = board.pieces(piece);
            if (piece_bb & sq_bb).0 != 0 {
                if (board.color_combined(Color::White) & sq_bb).0 != 0 {
                    white_value += get_piece_square_value(piece, square, Color::White, &phase);
                } else if (board.color_combined(Color::Black) & sq_bb).0 != 0 {
                    black_value += get_piece_square_value(piece, square, Color::Black, &phase);
                }
            }
        }
    }

    // Add material values and bonuses
    white_value += evaluate_material(board, Color::White, &phase);
    black_value += evaluate_material(board, Color::Black, &phase);

    // Add attack evaluation
    for square in 0..64 {
        white_value += evaluate_attacks(board, square, Color::White);
        black_value += evaluate_attacks(board, square, Color::Black);
    }

    // Add checkmate pattern detection
    white_value += detect_checkmate_patterns(board, Color::White);
    black_value += detect_checkmate_patterns(board, Color::Black);

    // Modify the final evaluation to be from the perspective of the side to move
    let score = match board.side_to_move() {
        Color::White => white_value - black_value,
        Color::Black => black_value - white_value,
    };

    score
}

fn evaluate_material(board: &Board, color: Color, phase: &GamePhase) -> f64 {
    let mut value = 0.0;

    // Count piece material
    let piece_counts = [
        (QUEEN, board.pieces(QUEEN) & board.color_combined(color)),
        (ROOK, board.pieces(ROOK) & board.color_combined(color)),
        (BISHOP, board.pieces(BISHOP) & board.color_combined(color)),
        (KNIGHT, board.pieces(KNIGHT) & board.color_combined(color)),
        (PAWN, board.pieces(PAWN) & board.color_combined(color)),
    ];

    for (piece, bb) in piece_counts {
        let count = bb.popcnt();
        match piece {
            Piece::Queen => {
                if count == 1 {
                    value += QUEEN_VALUE_NORMAL;
                } else if count > 1 {
                    value += QUEEN_VALUE_THRESHOLD_ADVANTAGE + QUEEN_VALUE_SECOND_QUEEN;
                }
            }
            Piece::Rook => {
                if count > 0 {
                    // Process first rook
                    let square = bb.0.trailing_zeros() as usize;
                    let mut info = analyze_rook_position(board, square, color);
                    info.is_first_rook = true;
                    value += get_rook_value(phase, true) + get_rook_position_bonus(&info);

                    if count > 1 {
                        // Process second rook
                        let second_bb = BitBoard(bb.0 & (bb.0 - 1)); // Clear least significant bit
                        let second_square = second_bb.0.trailing_zeros() as usize;
                        let mut info = analyze_rook_position(board, second_square, color);
                        info.is_first_rook = false;
                        value += get_rook_value(phase, false) + get_rook_position_bonus(&info);

                        // Add bonus for connected rooks
                        if (ROOK_ATTACKS[square].0 & second_bb.0) != 0 {
                            value += 0.2; // Connected rooks bonus
                        }
                    }
                }
            }
            Piece::Bishop => {
                value += count as f64 * BISHOP_VALUE;
                if count >= 2 {
                    value += get_bishop_pair_bonus(phase);
                }
            }
            Piece::Knight => value += count as f64 * get_knight_value(phase),
            Piece::Pawn => value += count as f64 * get_pawn_value(phase),
            _ => {}
        }
    }

    value
}
