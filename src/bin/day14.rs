use color_eyre::{
    eyre::{bail, eyre},
    Report, Result,
};

use std::{
    collections::HashMap,
    fmt::{Debug, Display, Write},
    io,
    str::FromStr,
};

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
struct Pos {
    x: isize,
    y: isize,
}

impl Pos {
    fn down(self) -> Pos {
        Pos {
            y: self.y + 1,
            ..self
        }
    }
    fn left(self) -> Pos {
        Pos {
            x: self.x - 1,
            ..self
        }
    }
    fn right(self) -> Pos {
        Pos {
            x: self.x + 1,
            ..self
        }
    }
    fn min(self, other: Pos) -> Pos {
        Pos {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
        }
    }
    fn max(self, other: Pos) -> Pos {
        Pos {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
        }
    }
}

impl FromStr for Pos {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (x, y) = s.split_once(',').ok_or_else(|| eyre!("expected comma"))?;
        let x = x.parse()?;
        let y = y.parse()?;
        Ok(Pos { x, y })
    }
}

fn parse_path(s: &str) -> Result<Vec<Pos>> {
    let mut out = vec![];
    for part in s.split(" -> ") {
        out.push(part.parse()?);
    }
    Ok(out)
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Tile {
    Air,
    Rock,
    SandFalling,
    SandResting,
}

impl Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tile::Air => f.write_char('.')?,
            Tile::Rock => f.write_char('#')?,
            Tile::SandFalling => f.write_char('+')?,
            Tile::SandResting => f.write_char('o')?,
        }
        Ok(())
    }
}

#[derive(Clone, Default)]
struct Sandbox {
    tiles: HashMap<Pos, Tile>,
    activesand: Option<Pos>,
    floor_y: Option<isize>,
}

impl Debug for Sandbox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mincoord = self.tiles.keys().copied().reduce(Pos::min).unwrap();
        let mut maxcoord = self.tiles.keys().copied().reduce(Pos::max).unwrap();
        if let Some(fy) = self.floor_y {
            maxcoord.y = fy;
        }
        writeln!(f, "{:?} -> {:?}", mincoord, maxcoord)?;
        for y in mincoord.y..=maxcoord.y {
            for x in mincoord.x..=maxcoord.x {
                let tile = self.tile_at(Pos { x, y });
                write!(f, "{}", tile)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Sandbox {
    fn fill_rock_path(&mut self, path: &[Pos]) -> Result<()> {
        let segments = path.windows(2);
        for segment in segments {
            if let [from, to] = segment {
                if from.x == to.x {
                    for y in from.y.min(to.y)..=from.y.max(to.y) {
                        self.tiles.insert(Pos { x: from.x, y }, Tile::Rock);
                    }
                } else if from.y == to.y {
                    for x in from.x.min(to.x)..=from.x.max(to.x) {
                        self.tiles.insert(Pos { x, y: from.y }, Tile::Rock);
                    }
                } else {
                    bail!("Path segment not vertical or horizontal")
                }
            } else {
                unreachable!()
            }
        }
        Ok(())
    }
    fn tile_at(&self, pos: Pos) -> Tile {
        match self.tiles.get(&pos) {
            None if Some(pos.y) == self.floor_y => Tile::Rock,
            None => Tile::Air,
            Some(&t) => t,
        }
    }
    fn tick_sand(&mut self) -> bool {
        if let Some(sp) = self.activesand.take() {
            assert!(self.tiles.remove(&sp) == Some(Tile::SandFalling));
            for check in [sp.down(), sp.down().left(), sp.down().right()] {
                if self.tile_at(check) == Tile::Air {
                    assert!(self.tiles.insert(check, Tile::SandFalling).is_none());
                    self.activesand = Some(check);
                    break;
                }
            }
            if self.activesand.is_none() {
                self.tiles.insert(sp, Tile::SandResting);
            }
        } else {
            let spos = Pos { x: 500, y: 0 };
            match self.tiles.get(&spos) {
                None => {
                    self.activesand = Some(spos);
                    self.tiles.insert(spos, Tile::SandFalling);
                }
                // blocked
                Some(Tile::SandResting) => return true,
                Some(t) => panic!("unexpected tile blocking inflow, {:?}", t),
            }
        }
        false
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let input = io::read_to_string(io::stdin())?;
    let do_part2 = std::env::var("PART2").is_ok();
    let paths: Vec<Vec<Pos>> = input.lines().map(parse_path).collect::<Result<_>>()?;
    let mut sandbox = Sandbox::default();
    for p in &paths {
        sandbox.fill_rock_path(p)?;
    }
    let mut maxy = sandbox.tiles.keys().map(|p| p.y).max().expect("max y");
    if do_part2 {
        sandbox.floor_y = Some(maxy + 2);
        maxy += 2;
    }
    for step in 1.. {
        let blocked = sandbox.tick_sand();

        let stop = blocked
            || if let Some(active) = sandbox.activesand {
                active.y > maxy
            } else {
                false
            };
        if stop {
            let rsu = sandbox
                .tiles
                .values()
                .filter(|t| **t == Tile::SandResting)
                .count();
            println!("Sandbox tick {}: {:?}", step, sandbox);
            println!("{} sand units came to rest.", rsu);
            break;
        }
    }
    Ok(())
}
