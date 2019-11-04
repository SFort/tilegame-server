#![allow(dead_code)]
use game;
use std::{
    io::prelude::*,
    net::TcpStream,
    sync::{Arc, RwLock},
};
//TODO prior to v1.0
//      reset map command
//      vote map
//      client defined maps
//      client defined auth key (replacment for static string turns)
//      client defined user names
//      support more players
//
pub fn handle_stream(
    mut stream: TcpStream,
    field: Arc<RwLock<Vec<Vec<game::Tile>>>>,
    state: Arc<RwLock<game::State>>,
) {
    let command: u8;
    let args: Vec<u8>;
    {
        let mut buffer = [0; 128];
        match stream.read(&mut buffer) {
            Ok(0) => return,
            Ok(_) => {
                command = buffer[0];
                let mut n = buffer.len() - 1;
                while buffer[n] == 0 {
                    n -= 1;
                }
                args = buffer[1..n + 1].into();
            }
            Err(e) => {
                println!("Handle stream failed to read\n{}", e);
                return;
            }
        }
    }

    if command > 15 {
        if let Some(victor) = &state.read().unwrap().victor {
            stream.write(&[3, 1, 2]).unwrap();
            stream.write(victor.as_bytes()).unwrap();
            return;
        }
    }
    if !state.read().unwrap().updated {
        game::table::update_mod(&mut field.write().unwrap(), &state.read().unwrap().turn);
        state.write().unwrap().updated = true;
    }
    match (command, args) {
        //get field HP
        (1, _) => {
            let field = field.read().unwrap();
            stream
                .write(&[2, field.len() as u8, field[0].len() as u8])
                .unwrap();
            let hp = game::table::to_hp(field.to_vec());
            stream.write(&[1]).unwrap();
            for row in hp {
                stream.write(&row).unwrap();
            }
        }
        //get ownership status
        (2, _) => {
            let field = field.read().unwrap();
            let mut buffer = vec![0; 128];
            stream.write(&[1, 10, 1]).unwrap();
            if let Ok(bytes) = stream.read(&mut buffer) {
                buffer.truncate(bytes);
            }
            stream
                .write(&[2, field.len() as u8, field[0].len() as u8])
                .unwrap();
            let val = game::table::to_occupant(&field.to_vec(), &buffer);
            stream.write(&[2]).unwrap();
            for row in val {
                stream.write(&row).unwrap();
            }
        }
        //get players turn
        (3, _) => {
            stream.write(&[3, 1, 1]).unwrap();
            stream
                .write(&state.read().unwrap().player_turn().as_bytes())
                .unwrap();
            return;
        }
        //Roll
        (16, _) => {
            if is_players_turn(&mut stream, &state) {
                let mut state = state.write().unwrap();
                if state.dice.is_none() {
                    state.roll_die();
                }
                if let Some(dice) = state.dice {
                    stream.write(&[1, 1, dice]).unwrap();
                } else {
                    stream.write(&[1, 2, 1]).unwrap();
                }
            } else {
                stream.write(&[0, 10, 16]).unwrap();
            }
        }
        //Coin flip
        (17, _) => {
            if is_players_turn(&mut stream, &state) {
                let mut state = state.write().unwrap();
                state.flip_coin();
                if let Some(dice) = state.dice {
                    stream.write(&[1, 3, dice]).unwrap();
                } else {
                    stream.write(&[1, 2, 2]).unwrap();
                }
            } else {
                stream.write(&[0, 10, 17]).unwrap();
            }
        } //Capture area

        (32, args) => {
            if args.len() < 5 {
                stream.write(&[0, 32, 2]).unwrap();
                return;
            }
            if !is_players_turn(&mut stream, &state) {
                stream.write(&[0, 10, 32]).unwrap();
                return;
            }

            let mut p1 = (0, 0);
            let mut p2 = (0, 0);
            if args[0] > args[2] {
                p1.0 = args[2] as usize;
                p2.0 = args[0] as usize;
            } else {
                p1.0 = args[0] as usize;
                p2.0 = args[2] as usize;
            };
            if args[1] > args[3] {
                p1.1 = args[3] as usize;
                p2.1 = args[1] as usize;
            } else {
                p1.1 = args[1] as usize;
                p2.1 = args[3] as usize;
            };
            p2.0 += 1;
            p2.1 += 1;
            if field.read().unwrap().len() < p2.0 || field.read().unwrap()[0].len() < p2.1 {
                stream.write(&[0, 32, 5]).unwrap();
                return;
            }
            if !is_dice_enough(&state, &field, args[4] != 1, (&p1, &p2)) {
                stream.write(&[0, 32, 3]).unwrap();
                return;
            }
            if is_cap_area_valid(&state, &field, (&p1, &p2)) {
                let mut state = state.write().unwrap();
                let mut field = field.write().unwrap();
                if game::table::has_won_area(&state.player_turn(), &field, &p1, &p2) {
                    stream.write(&[1, 32, 64]).unwrap();
                    state.victor = Some(state.player_turn().into());
                    return;
                }
                game::table::capture_area(
                    state.player_turn(),
                    args[4] != 1,
                    &state.turn,
                    &mut field,
                    &p1,
                    &p2,
                );
                state.next_turn();
                stream.write(&[1, 32, 1]).unwrap();
            } else {
                stream.write(&[0, 32, 4]).unwrap();
            }
        }
        _ => {
            stream.write(&[0, 255, 0]).unwrap();
        }
    };
}
fn is_dice_enough(
    state: &RwLock<game::State>,
    field: &RwLock<Vec<Vec<game::Tile>>>,
    tuffwalls: bool,
    p: (&(usize, usize), &(usize, usize)),
) -> bool {
    use *;
    let state = state.read().unwrap();
    let field = field.read().unwrap();
    match state.dice {
        None => false,
        Some(r) => {
            (if tuffwalls { r as u32 / 2 } else { r as u32 })
                >= game::table::sum_hp_area(&field, p.0, p.1)
        }
    }
}
fn is_cap_area_valid(
    state: &RwLock<game::State>,
    field: &RwLock<Vec<Vec<game::Tile>>>,
    p: (&(usize, usize), &(usize, usize)),
) -> bool {
    use *;
    let state = state.read().unwrap();
    let field = field.read().unwrap();
    game::table::is_based_area(
        &game::table::to_occupant(&field, state.player_turn().as_bytes()),
        p.0,
        p.1,
    )
}
fn is_players_turn(stream: &mut TcpStream, state: &RwLock<game::State>) -> bool {
    stream.write(&[1, 10, 1]).unwrap();
    let mut buffer = vec![0; 128];
    if let Ok(bytes) = stream.read(&mut buffer) {
        buffer.truncate(bytes);
    }
    state.read().unwrap().is_player_turn(&buffer[..])
}
