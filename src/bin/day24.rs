use color_eyre::Result;
use std::{
    collections::{HashSet, VecDeque},
    hash::Hash,
    io,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Dir {
    Up,
    Right,
    Down,
    Left,
}

impl Dir {
    fn flip(self) -> Dir {
        match self {
            Dir::Up => Dir::Down,
            Dir::Right => Dir::Left,
            Dir::Down => Dir::Up,
            Dir::Left => Dir::Right,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tile {
    Empty,
    Wall,
    Blizzard(Dir),
}

impl Tile {
    fn from_char(c: char) -> Option<Tile> {
        use Dir::*;
        use Tile::*;
        Some(match c {
            '.' => Empty,
            '#' => Wall,
            '^' => Blizzard(Up),
            'v' => Blizzard(Down),
            '>' => Blizzard(Right),
            '<' => Blizzard(Left),
            _ => None?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Pos(u16, u16);

impl Pos {
    fn go(self, dir: Dir) -> Option<Pos> {
        Some(match dir {
            Dir::Up => Pos(self.0, self.1.checked_sub(1)?),
            Dir::Right => Pos(self.0 + 1, self.1),
            Dir::Down => Pos(self.0, self.1 + 1),
            Dir::Left => Pos(self.0.checked_sub(1)?, self.1),
        })
    }
}

fn advance_blizzards(blizzards: &mut [(Pos, Dir)], walls: &HashSet<Pos>) {
    for (bpos, dir) in blizzards {
        let next = bpos.go(*dir).unwrap();
        let next = if walls.contains(&next) {
            let mut next = *bpos;
            let flip = dir.flip();
            loop {
                let back = next.go(flip).unwrap();
                if walls.contains(&back) {
                    break;
                } else {
                    next = back;
                }
            }
            next
        } else {
            next
        };
        *bpos = next;
    }
}

fn bloccupancy(blizzards: &[(Pos, Dir)]) -> HashSet<Pos> {
    blizzards.iter().map(|(l, _)| *l).collect()
}

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
struct State {
    time_elapsed: u32,
    blizzard_occupancy_idx: usize,
    pos: Pos,
    need_snack: bool,
    has_snack: bool,
}

impl State {
    fn next_states(
        &self,
        bloccupancies: &[HashSet<Pos>],
        walls: &HashSet<Pos>,
    ) -> impl Iterator<Item = State> {
        use Dir::*;
        let blen = bloccupancies.len();
        let curblocced = &bloccupancies[self.blizzard_occupancy_idx];
        let nidx = (self.blizzard_occupancy_idx + 1) % blen;
        let nextblocced = &bloccupancies[nidx];
        let alive = !curblocced.contains(&self.pos);
        if !alive {
            println!("Dead branch");
        }
        let can_go = |dir: Dir| {
            let next = match self.pos.go(dir) {
                Some(pos) => pos,
                None => return false,
            };
            !nextblocced.contains(&next) && !walls.contains(&next)
        };
        let mut c = [
            (alive && can_go(Right)).then(|| self.advance(Some(Right), blen)),
            (alive && can_go(Down)).then(|| self.advance(Some(Down), blen)),
            (alive && can_go(Up)).then(|| self.advance(Some(Up), blen)),
            (alive && can_go(Left)).then(|| self.advance(Some(Left), blen)),
            (alive && !nextblocced.contains(&self.pos)).then(|| self.advance(None, blen)),
        ];
        if self.need_snack && !self.has_snack {
            c[0..4].reverse()
        }
        c.into_iter().flatten()
    }
    fn advance(&self, dir: Option<Dir>, blen: usize) -> State {
        State {
            time_elapsed: self.time_elapsed + 1,
            blizzard_occupancy_idx: (self.blizzard_occupancy_idx + 1) % blen,
            pos: match dir {
                Some(dir) => self.pos.go(dir).unwrap(),
                None => self.pos,
            },
            ..*self
        }
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let input = io::read_to_string(io::stdin())?;
    let mut walls = HashSet::new();
    let mut blizzards = vec![];
    let mut start = None;
    let mut end = None;
    for (y, line) in input.lines().enumerate() {
        for (x, c) in line.chars().enumerate() {
            let pos = Pos(x as u16, y as u16);
            let tile = Tile::from_char(c).expect("bad tile");
            match tile {
                Tile::Wall => {
                    walls.insert(pos);
                }
                Tile::Blizzard(dir) => {
                    blizzards.push((pos, dir));
                }
                Tile::Empty => {
                    if start.is_none() {
                        start = Some(pos);
                    }
                    end = Some(pos);
                }
            };
        }
    }
    let start = start.unwrap();
    let end = end.unwrap();
    let mut bloccs = vec![];
    {
        let mut uniqbloccs = HashSet::new();
        loop {
            if !uniqbloccs.insert(blizzards.clone()) {
                break;
            }
            bloccs.push(bloccupancy(&blizzards[..]));
            advance_blizzards(&mut blizzards, &walls);
        }
    }
    println!("{start:?} -> {end:?}, blocc cycle: {}", bloccs.len());
    let mut q = VecDeque::new();
    q.push_back(State {
        time_elapsed: 0,
        blizzard_occupancy_idx: 0,
        pos: start,
        need_snack: false,
        has_snack: false,
    });
    let mut found = None;
    let mut visited = HashSet::new();
    while let Some(mut state) = q.pop_front() {
        if state.pos == end && !state.need_snack {
            state.need_snack = true;
        }
        if state.need_snack && state.pos == start {
            state.has_snack = true;
        }
        if visited.contains(&state) {
            continue;
        }
        visited.insert(state);
        let goal_reached = state.has_snack && state.pos == end;
        match found {
            None => {
                if goal_reached {
                    found = Some(state);
                    println!("found path: {:?}", found);
                }
            }
            Some(best) => {
                if state.time_elapsed > best.time_elapsed {
                    continue;
                }
                if goal_reached && best.time_elapsed > state.time_elapsed {
                    println!("better path: {:?}", found);
                    found = Some(state);
                }
            }
        }
        for next_state in state.next_states(&bloccs[..], &walls) {
            q.push_back(next_state);
        }
    }
    println!("best path: {:?}", found);
    Ok(())
}