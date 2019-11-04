#![allow(dead_code)]
use game::Tile;
pub fn new(width: usize, height: usize) -> Vec<Vec<Tile>> {
    vec![
        vec![
            Tile {
                effect: Vec::new(),
                owner: String::new(),
                base: false,
                hp: 1,
            };
            width
        ];
        height
    ]
}
pub fn update_mod(table: &mut Vec<Vec<Tile>>, turn: &u16) {
    for x in 0..table.len() {
        for y in 0..table[0].len() {
            table[x][y].update_mod(turn);
        }
    }
}
pub fn sum_hp_area(table: &Vec<Vec<Tile>>, p1: &(usize, usize), p2: &(usize, usize)) -> u32 {
    let mut result: u32 = 0;
    for x in p1.0..p2.0 {
        for y in p1.1..p2.1 {
            result += table[x][y].get_hp_sum() as u32;
        }
    }
    result
}
pub fn has_won_area(
    name: &str,
    table: &Vec<Vec<Tile>>,
    p1: &(usize, usize),
    p2: &(usize, usize),
) -> bool {
    for x in p1.0..p2.0 {
        for y in p1.1..p2.1 {
            if table[x][y].has_won(name) {
                return true;
            }
        }
    }
    false
}
pub fn capture_area(
    name: &str,
    reenforced: bool,
    time: &u16,
    table: &mut Vec<Vec<Tile>>,
    p1: &(usize, usize),
    p2: &(usize, usize),
) {
    for x in p1.0..p2.0 {
        for y in p1.1..p2.1 {
            if table[x][y].owner != String::new() {
                table[x][y].add_mod(time + 2, -1);
            }
            if reenforced {
                table[x][y].add_mod(time + 2, 2);
            }
            table[x][y].capture(name, reenforced);
        }
    }
}
pub fn to_hp(table: Vec<Vec<Tile>>) -> Vec<Vec<u8>> {
    table
        .iter()
        .map(|x| x.iter().map(|y| y.get_hp_sum()).collect::<Vec<u8>>())
        .collect::<Vec<Vec<u8>>>()
}
pub fn is_based_area(table: &Vec<Vec<u8>>, p1: &(usize, usize), p2: &(usize, usize)) -> bool {
    let mut table = table.clone();
    let mut should_loop = true;
    let mut is_based = false;
    for x in p1.0..p2.0 {
        for y in p1.1..p2.1 {
            table[x][y] = 3;
        }
    }
    'based: while should_loop {
        should_loop = false;
        for x in 0..table.len() {
            for y in 0..table[0].len() {
                if table[x][y] == 3 {
                    if y != 0 {
                        //check y-1
                        match table[x][y - 1] {
                            2 => {
                                table[x][y - 1] = 3;
                                should_loop = true;
                            }
                            1 => {
                                is_based = true;
                                break 'based;
                            }
                            _ => {}
                        }
                    }
                    if x != 0 {
                        //check x-1
                        match table[x - 1][y] {
                            2 => {
                                table[x - 1][y] = 3;
                                should_loop = true;
                            }
                            1 => {
                                is_based = true;
                                break 'based;
                            }
                            _ => {}
                        }
                    }
                    if x != table.len() - 1 {
                        //check x+1
                        match table[x + 1][y] {
                            2 => {
                                table[x + 1][y] = 3;
                                should_loop = true;
                            }
                            1 => {
                                is_based = true;
                                break 'based;
                            }
                            _ => {}
                        }
                    }
                    if y != table[0].len() - 1 {
                        //check y+1
                        match table[x][y + 1] {
                            2 => {
                                table[x][y + 1] = 3;
                                should_loop = true;
                            }
                            1 => {
                                is_based = true;
                                break 'based;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
    is_based
}
pub fn to_occupant(table: &Vec<Vec<Tile>>, name: &[u8]) -> Vec<Vec<u8>> {
    table
        .iter()
        .map(|x| {
            x.iter()
                .map(|y| match (y.base, y.owner.as_bytes()) {
                    (_, own) if own.is_empty() => 0,
                    (true, own) if own == name => 1,
                    (false, own) if own == name => 2,
                    (true, _) => 4,
                    (false, _) => 5,
                })
                .collect::<Vec<u8>>()
        })
        .collect::<Vec<Vec<u8>>>()
}
