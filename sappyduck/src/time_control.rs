// sappy: borrowed from walleye: https://github.com/MitchelPaulin/Walleye
use crate::movegen::Position;
use crate::movepick::pick_move;
use chess::Color;
use std::time::Instant;

pub const SAFEGUARD: f64 = 100.0; // msecs
const GAME_LENGTH: u32 = 30; // moves
const MAX_USAGE: f64 = 0.8; // percentage
const NO_TIME: u128 = 0;

pub struct GameTime {
    // all time is in ms unless otherwise specified
    pub wtime: i128,
    pub btime: i128,
    pub winc: i128,
    pub binc: i128,
    pub movestogo: Option<u32>,
}

/*
    Big thanks to @mvanthoor (https://github.com/mvanthoor) whose chess engine
    the below time control implementation was adapted from
*/
impl GameTime {
    // Calculates the time the engine allocates for searching a single
    // move. This depends on the number of moves still to go in the game.
    pub fn calculate_time(&self, color: Color) -> u128 {
        let mtg = self.movestogo.unwrap_or(GAME_LENGTH) as f64;
        let is_white = color == Color::White;
        let clock = if is_white { self.wtime } else { self.btime } as f64;
        let increment = if is_white { self.winc } else { self.binc } as f64;
        let base_time = clock - SAFEGUARD;

        // return a time slice.
        if base_time <= 0.0 {
            if increment > 0.0 {
                (increment * MAX_USAGE).round() as u128
            } else {
                NO_TIME
            }
        } else {
            (base_time * MAX_USAGE / mtg).round() as u128
        }
    }
}

pub fn pick_move_timed(position: &mut Position, time_slice: u128) -> Option<String> {
    // Placeholder for move picking logic with time control
    // Implement your move picking logic here
    // For now, just return the first legal move
    let start_time = Instant::now();
    while start_time.elapsed().as_millis() < time_slice {
        // Simulate thinking process
    }
    pick_move(position)
}
