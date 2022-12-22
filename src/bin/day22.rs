use color_eyre::Result;
use num::integer::Roots;
use regex::Regex;
use std::{collections::HashSet, hash::Hash, io};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct Loc(usize, usize);

#[derive(Default, Clone)]
struct Board {
    height: usize,
    width: usize,
    open_tiles: HashSet<Loc>,
    walls: HashSet<Loc>,
}

impl Board {
    fn square_edge(&self) -> usize {
        let total_tiles = self.open_tiles.len() + self.walls.len();
        (total_tiles / 6).sqrt()
    }
    fn face_corners(&self) -> [Loc; 6] {
        let elen = self.square_edge();
        let xtiles = self.width / elen;
        let ytiles = self.height / elen;
        let mut facecorners = [Loc(0, 0); 6];
        let mut cidx = 0;
        for y in 0..ytiles {
            for x in 0..xtiles {
                let corner = Loc(x * elen, y * elen);
                if self.open_tiles.contains(&corner) || self.walls.contains(&corner) {
                    facecorners[cidx] = corner;
                    cidx += 1;
                }
            }
        }
        assert!(cidx == 6);
        facecorners
    }
}

impl Board {
    fn try_advance(&self, from: Loc, dir: Dir) -> Option<Loc> {
        if let Some(nowrap) = (|| {
            Some(match dir {
                Dir::Right if from.0 < self.width => Loc(from.0 + 1, from.1),
                Dir::Down if from.1 < self.height => Loc(from.0, from.1 + 1),
                Dir::Left => Loc(from.0.checked_sub(1)?, from.1),
                Dir::Up => Loc(from.0, from.1.checked_sub(1)?),
                _ => None?,
            })
        })() {
            if self.open_tiles.contains(&nowrap) {
                return Some(nowrap);
            }
            if self.walls.contains(&nowrap) {
                return None;
            }
        }
        // must be wrapping
        let all_tiles: HashSet<_> = self.open_tiles.union(&self.walls).collect();
        let wrapto = match dir {
            Dir::Right => all_tiles
                .into_iter()
                .filter(|l| l.1 == from.1)
                .min_by_key(|l| l.0)
                .copied(),
            Dir::Down => all_tiles
                .into_iter()
                .filter(|l| l.0 == from.0)
                .min_by_key(|l| l.1)
                .copied(),
            Dir::Left => all_tiles
                .into_iter()
                .filter(|l| l.1 == from.1)
                .max_by_key(|l| l.0)
                .copied(),
            Dir::Up => all_tiles
                .into_iter()
                .filter(|l| l.0 == from.0)
                .max_by_key(|l| l.1)
                .copied(),
        }
        .expect("Wrap to");
        if self.open_tiles.contains(&wrapto) {
            return Some(wrapto);
        }
        if self.walls.contains(&wrapto) {
            return None;
        }
        panic!("wrap to nowhere")
    }
}

#[derive(Debug, Copy, Clone)]
enum Dir {
    Right = 0,
    Down = 1,
    Left = 2,
    Up = 3,
}

impl Dir {
    fn turn_right(self) -> Dir {
        use Dir::*;
        match self {
            Up => Right,
            Right => Down,
            Down => Left,
            Left => Up,
        }
    }
    fn turn_left(self) -> Dir {
        use Dir::*;
        match self {
            Up => Left,
            Left => Down,
            Down => Right,
            Right => Up,
        }
    }
}

#[derive(Debug)]
struct Player {
    location: Loc,
    facing: Dir,
}

#[derive(Copy, Clone, Debug)]
enum Cmd {
    Forward(usize),
    TurnRight,
    TurnLeft,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let input = io::read_to_string(io::stdin())?;
    let (boardtext, given_path) = input.split_once("\n\n").expect("no separator line");
    let max_x = boardtext.lines().map(str::len).max().unwrap();
    let max_y = boardtext.lines().count();
    let mut board = Board {
        height: max_y,
        width: max_x,
        ..Default::default()
    };
    for (y, bline) in boardtext.lines().enumerate() {
        for (x, c) in bline.chars().enumerate() {
            let loc = Loc(x, y);
            match c {
                '.' => {
                    board.open_tiles.insert(loc);
                }
                '#' => {
                    board.walls.insert(loc);
                }
                _ => (),
            };
        }
    }
    let cmdre = Regex::new(r"(\d+)|(R|L)").unwrap();
    let mut cmds: Vec<Cmd> = vec![];
    for cmdcap in cmdre.captures_iter(given_path) {
        if let Some(fwd_dist) = cmdcap.get(1) {
            cmds.push(Cmd::Forward(fwd_dist.as_str().parse()?));
        } else if let Some(turn_dir) = cmdcap.get(2) {
            match turn_dir.as_str() {
                "R" => cmds.push(Cmd::TurnRight),
                "L" => cmds.push(Cmd::TurnLeft),
                _ => unreachable!(),
            };
        }
    }

    let mut player = Player {
        location: board
            .open_tiles
            .iter()
            .min_by_key(|l| (l.1, l.0))
            .copied()
            .expect("topleft open tile"),
        facing: Dir::Right,
    };

    println!("start: {player:?}");
    for cmd in &cmds {
        //println!("{cmd:?}");
        match cmd {
            Cmd::Forward(dist) => {
                for _ in 0..*dist {
                    if let Some(adv) = board.try_advance(player.location, player.facing) {
                        player.location = adv;
                        //println!(" move to {adv:?}");
                    } else {
                        //println!(" blocked");
                        break;
                    }
                }
            }
            Cmd::TurnRight => {
                player.facing = player.facing.turn_right();
                //println!(" facing: {:?}", player.facing);
            }
            Cmd::TurnLeft => {
                player.facing = player.facing.turn_left();
                //println!(" facing: {:?}", player.facing);
            }
        }
    }

    //println!("end: {player:?}");
    let passwd =
        1000 * (player.location.1 + 1) + 4 * (player.location.0 + 1) + player.facing as usize;
    println!("passwd: {passwd}");

    let edgelen = board.square_edge();
    let faces = board.face_corners();
    println!("part2, folding. edgelen: {edgelen}, faces at: {faces:?}");
    Ok(())
}
