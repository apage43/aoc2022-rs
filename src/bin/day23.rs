use color_eyre::Result;
use std::{
    collections::{HashMap, HashSet},
    io,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Loc(isize, isize);

impl Loc {
    fn go(self, dir: Dir) -> Loc {
        use Dir::*;
        match dir {
            N => Loc(self.0, self.1 - 1),
            S => Loc(self.0, self.1 + 1),
            E => Loc(self.0 + 1, self.1),
            W => Loc(self.0 - 1, self.1),
        }
    }
    fn adjacents(self) -> [Loc; 8] {
        use Dir::*;
        [
            self.go(N).go(E),
            self.go(N),
            self.go(N).go(W),
            self.go(W),
            self.go(W).go(S),
            self.go(S),
            self.go(S).go(E),
            self.go(E),
        ]
    }
    fn propspace(self, dir: Dir) -> [Loc; 3] {
        use Dir::*;
        match dir {
            N => [self.go(N), self.go(N).go(E), self.go(N).go(W)],
            S => [self.go(S), self.go(S).go(E), self.go(S).go(W)],
            E => [self.go(E), self.go(E).go(S), self.go(E).go(N)],
            W => [self.go(W), self.go(W).go(S), self.go(W).go(N)],
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum Dir {
    N,
    S,
    E,
    W,
}

fn print_grid(elflocs: &HashSet<Loc>) {
    let mut minloc = Loc(isize::MAX, isize::MAX);
    let mut maxloc = Loc(isize::MIN, isize::MIN);
    for loc in elflocs {
        minloc.0 = minloc.0.min(loc.0);
        minloc.1 = minloc.1.min(loc.1);
        maxloc.0 = maxloc.0.max(loc.0);
        maxloc.1 = maxloc.1.max(loc.1);
    }
    for y in minloc.1..=maxloc.1 {
        for x in minloc.0..=maxloc.0 {
            if elflocs.contains(&Loc(x, y)) {
                print!("#");
            } else {
                print!(".");
            }
        }
        println!();
    }
}

fn empty_surface(elflocs: &HashSet<Loc>) -> usize {
    let mut empties = 0;
    let mut minloc = Loc(isize::MAX, isize::MAX);
    let mut maxloc = Loc(isize::MIN, isize::MIN);
    for loc in elflocs {
        minloc.0 = minloc.0.min(loc.0);
        minloc.1 = minloc.1.min(loc.1);
        maxloc.0 = maxloc.0.max(loc.0);
        maxloc.1 = maxloc.1.max(loc.1);
    }
    for y in minloc.1..=maxloc.1 {
        for x in minloc.0..=maxloc.0 {
            if !elflocs.contains(&Loc(x, y)) {
                empties += 1;
            }
        }
    }
    empties
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let input = io::read_to_string(io::stdin())?;
    let mut elflocs: HashSet<Loc> = Default::default();
    for (y, line) in input.lines().enumerate() {
        for (x, c) in line.chars().enumerate() {
            if c == '#' {
                elflocs.insert(Loc(x as isize, y as isize));
            }
        }
    }
    print_grid(&elflocs);
    let mut proposals: HashMap<Loc, Vec<Loc>> = Default::default();
    let mut propdirs = [Dir::N, Dir::S, Dir::W, Dir::E];
    for step in 1.. {
        for elfloc in &elflocs {
            if elfloc.adjacents().iter().any(|l| elflocs.contains(l)) {
                for dir in propdirs.iter() {
                    if elfloc.propspace(*dir).iter().all(|l| !elflocs.contains(l)) {
                        let dest = elfloc.go(*dir);
                        proposals.entry(dest).or_default().push(*elfloc);
                        break;
                    }
                }
            }
        }
        if proposals.is_empty() {
            print_grid(&elflocs);
            println!("No moves at step {step}");
            break;
        }
        for (dest, srcs) in proposals.drain() {
            if srcs.len() == 1 {
                let src = srcs.first().unwrap();
                elflocs.remove(src);
                elflocs.insert(dest);
            }
        }
        propdirs.rotate_left(1);
        if step == 10 {
            print_grid(&elflocs);
            println!("empties after step 10: {}", empty_surface(&elflocs));
        }
    }

    Ok(())
}
