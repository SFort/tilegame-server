use game::table;
use std::{
    net::TcpListener,
    sync::{Arc, RwLock},
    thread,
};

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
mod game;
mod net;
fn main() {
    println!("TileGame Server V:.{}", CURRENT_VERSION);
    let listener = TcpListener::bind("0.0.0.0:1760").expect("Failed bind");

    let field = Arc::new(RwLock::new(table::new(10, 18)));
    let state = Arc::new(RwLock::new(game::State::new()));
    {
        let table = field.clone();
        let mut field = table.write().unwrap();

        for i in 0..field[0].len() {
            field[0][i].set_base("fort".into());
            field[17][i].set_base("dog".into());
            let len = field.len();
            field[if i % 2 == 0 { len / 2 + 1 } else { len / 2 - 2 }][i].set_hp(4);
        }
    }
    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            let field = field.clone();
            let state = state.clone();
            thread::spawn(move || {
                net::handle_stream(stream, field, state);
            });
        }
    } //End of stream
} //End of main
