use std::collections::HashSet;

use color_eyre::{
    eyre::{bail, ContextCompat},
    Result,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Motion {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct Loc {
    x: i32,
    y: i32,
}

impl Loc {
    fn new(x: i32, y: i32) -> Loc {
        Loc { x, y }
    }
    fn moved(self, dir: Motion, dist: i32) -> Loc {
        use Motion::*;
        match dir {
            Up => Loc {
                x: self.x,
                y: self.y - dist,
            },
            Down => Loc {
                x: self.x,
                y: self.y + dist,
            },
            Left => Loc {
                x: self.x - dist,
                y: self.y,
            },
            Right => Loc {
                x: self.x + dist,
                y: self.y,
            },
        }
    }
    fn within(self, other: Loc, dist: i32) -> bool {
        (self.x - other.x).abs() <= dist && (self.y - other.y).abs() <= dist
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let num_knots = if std::env::args().any(|x| x.contains("part2")) {
        10
    } else {
        2
    };
    let mut moves = Vec::new();
    for line in std::io::stdin().lines() {
        let line = line?;
        let mut splits = line.split_whitespace();
        let dir = match splits.next().context("move direction")? {
            "U" => Motion::Up,
            "D" => Motion::Down,
            "L" => Motion::Left,
            "R" => Motion::Right,
            _ => bail!("bad move direction"),
        };
        let distance: i32 = splits.next().context("move length")?.parse()?;
        moves.push((dir, distance));
    }
    let mut rope = vec![Loc::new(0, 0); num_knots];
    let mut tailpath = Vec::new();
    tailpath.push(rope.last().unwrap().clone());
    for (dir, dist) in moves {
        for _m in 0..dist {
            rope[0] = rope[0].moved(dir, 1);
            for segment in 1..rope.len() {
                if !rope[segment].within(rope[segment - 1], 1) {
                    let newloc = Loc {
                        x: rope[segment].x + (rope[segment - 1].x - rope[segment].x).signum(),
                        y: rope[segment].y + (rope[segment - 1].y - rope[segment].y).signum(),
                    };
                    if segment == rope.len() - 1 {
                        tailpath.push(newloc.clone())
                    }
                    rope[segment] = newloc;
                }
            }
        }
    }
    let mut tail_uniqs = HashSet::new();
    tail_uniqs.extend(tailpath.iter().cloned());
    println!("Unique tail positions: {}", tail_uniqs.len());
    Ok(())
}
