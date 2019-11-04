#![allow(dead_code)]
use std::time::SystemTime;
pub mod table;
impl Tile {
    pub fn update_mod(&mut self, turn: &u16) {
        self.effect = self
            .effect
            .drain(..)
            .filter(|x| &x.expires >= turn)
            .collect::<Vec<Effect>>();
    }
    pub fn add_mod(&mut self, expires: u16, modifier: i8) {
        self.effect.push(Effect { expires, modifier });
    }
    pub fn get_hp_sum(&self) -> u8 {
        match (self.hp as i16).checked_add(self.get_mod() as i16) {
            Some(r) if r > 0 => r as u8,
            _ => 0,
        }
    }
    pub fn get_mod(&self) -> i8 {
        self.effect.iter().map(|e: &Effect| e.modifier).sum()
    }
    pub fn set_hp(&mut self, hp: u8) {
        self.hp = hp;
    }
    pub fn set_base(&mut self, owner: String) {
        self.owner = owner;
        self.base = true;
        self.hp = 3;
    }
    pub fn has_won(&self, name: &str) -> bool {
        (!(self.owner == name.to_string() || self.owner == String::new())) && self.base
    }
    pub fn capture(&mut self, owner: &str, double: bool) {
        self.owner = owner.into();
        self.hp = if double { 3 } else { 2 };
    }
}
impl State {
    pub fn new() -> State {
        State {
            victor: None,
            player_turn: "dog".to_owned(),
            dice: None,
            dice_bonus: 1,
            flipped: false,
            updated: true,
            turn: 0,
            poor_rand: SystemTime::now(),
        }
    }
    pub fn next_turn(&mut self) {
        self.player_turn = if self.player_turn == "dog" {
            "fort".into()
        } else {
            "dog".into()
        };
        self.dice = None;
        self.dice_bonus = !self.flipped as u8;
        self.flipped = false;
        self.updated = false;
        self.turn += 1;
    }
    pub fn roll_die(&mut self) {
        self.dice = Some(die_rand(self.poor_rand) + self.dice_bonus);
        self.dice_bonus = 0;
        if self.dice.unwrap() == 0 {
            self.flipped = true;
            self.next_turn();
        }
    }
    pub fn flip_coin(&mut self) {
        if !self.flipped && self.dice.is_some() {
            self.dice = Some(flip_rand(self.poor_rand) * self.dice.unwrap());
            self.flipped = true;
            if self.dice.unwrap() == 0 {
                self.next_turn();
            }
        }
    }
    pub fn player_turn(&self) -> &str {
        &self.player_turn
    }
    pub fn is_player_turn(&self, name: &[u8]) -> bool {
        self.player_turn.as_bytes() == name
    }
}
fn die_rand(past: SystemTime) -> u8 {
    match SystemTime::now().duration_since(past).unwrap().as_millis() % 6 {
        1 => 6,
        2 | 3 => 2,
        4 | 5 => 4,
        _ => 0,
    }
}
fn flip_rand(past: SystemTime) -> u8 {
    match SystemTime::now().duration_since(past).unwrap().as_millis() % 3 {
        1 => 2,
        2 => 1,
        _ => 0,
    }
}
#[derive(Clone)]
pub struct State {
    pub victor: Option<String>,
    pub updated: bool,
    pub dice: Option<u8>,
    pub turn: u16,
    player_turn: String,
    dice_bonus: u8,
    flipped: bool,
    poor_rand: SystemTime,
}
#[derive(Clone)]
pub struct Tile {
    effect: Vec<Effect>,
    owner: String,
    base: bool,
    hp: u8,
}
#[derive(Clone, Debug)]
pub struct Effect {
    expires: u16,
    modifier: i8,
}
