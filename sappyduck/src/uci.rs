use crate::defs::FEN_START;
use crate::movegen::Position;
use crate::movepick::{alpha_beta_search, evaluate_board, pick_move, SearchParams}; // Added alpha_beta_search
use crate::time_control::{pick_move_timed, GameTime};
use chess::Color;
use std::io::{self, BufRead};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

// Add a static stop flag
static STOP_FLAG: AtomicBool = AtomicBool::new(false);

// Communicates with the Universal Chess Interface (UCI)
pub fn uci_loop() {
    let mut position = Position::from_fen(FEN_START);
    let mut game_time = GameTime {
        wtime: 0,
        btime: 0,
        winc: 0,
        binc: 0,
        movestogo: None,
    };
    let stdin = io::stdin();
    let mut input = String::new();

    loop {
        input.clear();
        stdin.lock().read_line(&mut input).unwrap();
        let command = input.trim();

        match command {
            // UCI protocol commands
            "uci" => {
                println!("id name SappyDuck");
                println!("id author sappyduckie");
                println!("uciok");
            }
            "isready" => {
                println!("readyok");
            }
            "ucinewgame" => {
                position = Position::from_fen(FEN_START);
            }
            cmd if cmd.starts_with("position startpos moves") => {
                position = Position::from_fen(FEN_START);
                let moves = &cmd[20..];
                for mv in moves.split_whitespace() {
                    position.make_move(mv);
                }
            }
            // Plug in the FEN string
            cmd if cmd.starts_with("position fen ") => {
                let fen = &cmd[13..];
                position = Position::from_fen(fen);
            }
            // Analyze the position to a certain depth
            cmd if cmd.starts_with("go depth ") => {
                let depth = cmd[9..].trim().parse().unwrap_or(1);
                println!("info string starting search at depth {}", depth);

                // Reset stop flag at start of search
                STOP_FLAG.store(false, Ordering::SeqCst);

                if let Some(best_move) = analyze_position(&mut position, depth) {
                    println!("bestmove {}", best_move);
                } else {
                    // Fallback to any legal move if no best move found
                    if let Some(first_move) = position.generate_legal_moves().first() {
                        println!("bestmove {}", first_move);
                    } else {
                        println!("info string no legal moves available");
                        println!("bestmove 0000"); // Standard "null move" notation
                    }
                }
            }
            // Analyze a position for a certain amount of time
            cmd if cmd.starts_with("go") => {
                STOP_FLAG.store(false, Ordering::SeqCst);
                if cmd.contains("infinite") {
                    let mut params = SearchParams::default();
                    params.max_time = Duration::from_secs(3600); // 1 hour for infinite analysis
                    let best_move = pick_move(&mut position);
                    if let Some(best_move) = best_move {
                        println!("bestmove {}", best_move);
                    } else {
                        println!("bestmove a1a1"); // Null move as fallback
                    }
                } else {
                    parse_go(cmd, &mut game_time);
                    let time_slice = game_time.calculate_time(position.board.side_to_move());
                    let start_time = Instant::now();
                    let best_move = pick_move_timed(&mut position, time_slice);
                    let elapsed_time = start_time.elapsed();
                    if let Some(best_move) = best_move {
                        println!("bestmove {} (time spent: {:?})", best_move, elapsed_time);
                    } else {
                        println!("bestmove (none) (time spent: {:?})", elapsed_time);
                    }
                }
            }
            "stop" => {
                STOP_FLAG.store(true, Ordering::SeqCst);
            }
            "quit" => {
                std::process::exit(0);
            }
            _ => {}
        }
    }
}

// For now, just pick a move
fn analyze_position(position: &mut Position, depth: u32) -> Option<String> {
    let mut params = SearchParams::default();
    params.max_time = Duration::from_secs(300); // 5 minutes max per analysis

    // Force depth to 1 regardless of input
    let max_depth = 1;
    let mut best_move = None;
    let mut best_score = f64::NEG_INFINITY;

    println!("info string starting analysis at depth {}", max_depth);

    // Generate moves first to check if any are available
    let legal_moves = position.generate_legal_moves();
    if legal_moves.is_empty() {
        println!("info string no legal moves in position");
        return None;
    }

    for current_depth in 1..=max_depth {
        params.depth = current_depth;
        params.start_time = Instant::now();

        let (score, mv) = alpha_beta_search(
            position,
            current_depth,
            f64::NEG_INFINITY,
            f64::INFINITY,
            position.board.side_to_move() == Color::White,
            &mut params,
        );

        if mv.is_some() {
            best_move = mv;
            best_score = score;
            println!(
                "info depth {} score cp {} nodes {} time {} pv {}",
                current_depth,
                (best_score * 100.0) as i32,
                params.nodes,
                params.start_time.elapsed().as_millis(),
                best_move.as_ref().unwrap()
            );
        }
    }

    best_move
}

// Parse the go command for time control
fn parse_go(cmd: &str, game_time: &mut GameTime) {
    let tokens: Vec<&str> = cmd.split_whitespace().collect();
    let mut i = 1;
    while i < tokens.len() {
        match tokens[i] {
            // White time control
            "wtime" => {
                if i + 1 < tokens.len() {
                    game_time.wtime = tokens[i + 1].parse().unwrap_or(0);
                }
            }
            // Black time control
            "btime" => {
                if i + 1 < tokens.len() {
                    game_time.btime = tokens[i + 1].parse().unwrap_or(0);
                }
            }
            // White increment
            "winc" => {
                if i + 1 < tokens.len() {
                    game_time.winc = tokens[i + 1].parse().unwrap_or(0);
                }
            }
            // Black increment
            "binc" => {
                if i + 1 < tokens.len() {
                    game_time.binc = tokens[i + 1].parse().unwrap_or(0);
                }
            }
            // Moves to go
            "movestogo" => {
                if i + 1 < tokens.len() {
                    game_time.movestogo = Some(tokens[i + 1].parse().unwrap_or(0));
                }
            }
            _ => {}
        }
        i += 2;
    }
}

// Add stop flag accessor
pub fn should_stop() -> bool {
    STOP_FLAG.load(Ordering::SeqCst)
}
