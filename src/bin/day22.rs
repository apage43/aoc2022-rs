use color_eyre::Result;
use num::integer::Roots;
use regex::Regex;
use std::{collections::HashSet, hash::Hash, io, ops::Add};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct Loc(usize, usize);

impl Add for Loc {
    type Output = Loc;

    fn add(self, rhs: Self) -> Self::Output {
        Loc(self.0 + rhs.0, self.1 + rhs.1)
    }
}

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

struct CubicBoard {
    board: Board,
    sqside: usize,
    corners: [Loc; 6],
}

impl CubicBoard {
    fn facenum(&self, loc: Loc) -> Option<usize> {
        for (idx, corner) in self.corners.iter().enumerate() {
            let endc = *corner + Loc(self.sqside, self.sqside);
            if loc.0 >= corner.0 && loc.0 < endc.0 && loc.1 >= corner.1 && loc.1 < endc.1 {
                return Some(idx);
            }
        }
        println!("No face: {:?}", loc);
        None
    }
    fn edgemap(&self, srcface: usize, fromdir: Dir) -> Option<(usize, Dir, bool)> {

        use Dir::*;
        if self.sqside == 4 {
            // example input
            Some(match (srcface, fromdir) {
                (0, Up) => (1, Down, true),
                (0, Right) => (5, Left, true),
                (0, Left) => (2, Down, false),
                (1, Up) => (0, Up, true),
                (1, Left) => (5, Up, true),
                (1, Down) => (4, Up, true),
                (2, Up) => (0, Right, false),
                (2, Down) => (4, Right, true),
                (3, Right) => (5, Down, true),
                (4, Left) => (2, Up, true),
                (4, Down) => (1, Up, true),
                (5, Down) => (1, Right, true),
                (5, Right) => (0, Left, true),
                _ => None?,
            })
        } else {
            // hardcoded for my input
            //  01
            //  2
            // 34
            // 5
            Some(match (srcface, fromdir) {
                // new face, new dir, flipped?
                (0, Up) => (5, Right, false),
                (0, Left) => (3, Right, true),
                (1, Up) => (5, Up, false),
                (1, Right) => (4, Left, true),
                (1, Down) => (2, Left, false),
                (2, Left) => (3, Down, false),
                (2, Right) => (1, Up, false),
                (3, Left) => (0, Right, true),
                (3, Up) => (2, Right, false),
                (4, Right) => (1, Left, true),
                (4, Down) => (5, Left, false),
                (5, Left) => (0, Down, false),
                (5, Right) => (4, Up, false),
                (5, Down) => (1, Down, false),
                _ => None?,
            })
        }
    }
    fn try_advance(&self, from: Loc, dir: Dir) -> Option<(Loc, Dir)> {
        if let Some(nowrap) = (|| {
            Some(match dir {
                Dir::Right if from.0 < self.board.width => Loc(from.0 + 1, from.1),
                Dir::Down if from.1 < self.board.height => Loc(from.0, from.1 + 1),
                Dir::Left => Loc(from.0.checked_sub(1)?, from.1),
                Dir::Up => Loc(from.0, from.1.checked_sub(1)?),
                _ => None?,
            })
        })() {
            if self.board.open_tiles.contains(&nowrap) {
                return Some((nowrap, dir));
            }
            if self.board.walls.contains(&nowrap) {
                return None;
            }
        }
        // walked off an edge
        let edgepos = match dir {
            Dir::Up | Dir::Down => from.0 % self.sqside,
            Dir::Right | Dir::Left => from.1 % self.sqside,
        };
        let (to, ndir, flip) = self
            .edgemap(self.facenum(from).unwrap(), dir)
            .expect("edge map");
        let edgepos = if flip {
            (self.sqside - 1) - edgepos
        } else {
            edgepos
        };
        let inner = match ndir {
            Dir::Right => Loc(0, edgepos),
            Dir::Down => Loc(edgepos, 0),
            Dir::Left => Loc(self.sqside - 1, edgepos),
            Dir::Up => Loc(edgepos, self.sqside - 1),
        };
        //println!( "moving from face {:?} -> {:?}, {:?} -> {:?}, {:?} -> {:?}", self.facenum(from), to, dir, ndir, from, inner );
        let nloc = self.corners[to] + inner;
        if self.board.walls.contains(&nloc) {
            None
        } else {
            Some((nloc, ndir))
        }
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
    let start_loc = board
        .open_tiles
        .iter()
        .min_by_key(|l| (l.1, l.0))
        .copied()
        .expect("topleft open tile");
    let mut player = Player {
        location: start_loc,
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

    let cubic = CubicBoard {
        sqside: edgelen,
        corners: faces,
        board,
    };
    println!("starting face: {:?}", cubic.facenum(start_loc));
    let mut player = Player {
        location: start_loc,
        facing: Dir::Right,
    };

    println!("start: {player:?}");
    for cmd in &cmds {
        //println!("{cmd:?}");
        match cmd {
            Cmd::Forward(dist) => {
                for _ in 0..*dist {
                    if let Some(adv) = cubic.try_advance(player.location, player.facing) {
                        (player.location, player.facing) = adv;
                        // print!(" -> {adv:?}");
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
    let passwd =
        1000 * (player.location.1 + 1) + 4 * (player.location.0 + 1) + player.facing as usize;
    println!("passwd: {passwd}");
    Ok(())
}
