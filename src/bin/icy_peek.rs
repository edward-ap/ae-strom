//! Port of mini-radio's icy_peek: headers, raw ICY blocks and changed titles.
//! Uses Strom's cancellable transport/parser; never opens an audio device.
#[path = "../diagnostics/icy_console.rs"]
mod icy_console;

fn main() {
    if let Err(error) = icy_console::run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
