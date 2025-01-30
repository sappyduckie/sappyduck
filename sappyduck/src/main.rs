extern crate chess;

mod bitboard;
mod defs;
mod movegen;
mod movepick;
mod time_control;
mod uci;

use uci::uci_loop;

fn main() {
    uci_loop();
}
