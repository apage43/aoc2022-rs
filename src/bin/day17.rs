use color_eyre::Result;
use core::panic;
use std::{
    collections::HashSet,
    fmt::{Display, Write},
    io,
    ops::{Add, Neg, Sub},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Push {
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Rock {
    w: usize,
    h: usize,
    occupancy: Vec<bool>,
}

impl Rock {
    fn blocked(&self, loc: Loc) -> bool {
        assert!(!loc.0.is_negative());
        assert!(!loc.1.is_negative());
        let rawloc = loc.1 as usize * self.w + loc.0 as usize;
        let rawloc = rawloc;
        if self.occupancy.len() > rawloc {
            self.occupancy[rawloc]
        } else {
            false
        }
    }
    fn from(def: &str) -> Rock {
        let inpls: Vec<&str> = def.trim().lines().map(str::trim).collect();
        let w = inpls[0].len();
        let h = inpls.len();
        let occupancy = def
            .chars()
            .filter(|&c| c == '.' || c == '#')
            .map(|c| c == '#')
            .collect();
        Rock { h, w, occupancy }
    }
}

impl Display for Rock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.h {
            for x in 0..self.w {
                f.write_char(if self.occupancy[y * self.w + x] {
                    '#'
                } else {
                    '.'
                })?;
            }
            writeln!(f)?
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
struct Loc(isize, isize);
impl Sub for Loc {
    type Output = Loc;

    fn sub(self, rhs: Self) -> Self::Output {
        Loc(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl Add for Loc {
    type Output = Loc;

    fn add(self, rhs: Self) -> Self::Output {
        Loc(self.0 + rhs.0, self.1 + rhs.1)
    }
}
const CHAMBER_WIDTH: isize = 7;

#[derive(Default)]
struct Chamber {
    falling_rock: Option<(Rock, Loc)>,
    occupancy: Vec<bool>,
}

impl Chamber {
    fn spawn_rock(&mut self, rock: Rock) {
        let x = 2;
        let y = self.stack_height().neg() - 3 - rock.h as isize;

        self.falling_rock = Some((rock, Loc(x, y)));
    }
    fn push_rock(&mut self, push: Push) {
        if let Some((rock, loc)) = self.falling_rock.take() {
            let nloc = Loc(
                loc.0
                    + match push {
                        Push::Left => -1,
                        Push::Right => 1,
                    },
                loc.1,
            );
            if nloc.0 + rock.w as isize > CHAMBER_WIDTH || nloc.0 < 0 {
                self.falling_rock = Some((rock, loc));
            } else {
                for y in 0..rock.h {
                    for x in 0..rock.w {
                        let iloc = Loc(x as isize, y as isize);
                        if nloc.1 + rock.h as isize > 0
                            || (rock.blocked(iloc) && self.blocked(nloc + iloc))
                        {
                            self.falling_rock = Some((rock, loc));
                            return;
                        }
                    }
                }
                self.falling_rock = Some((rock, nloc));
            }
        }
    }
    fn drop_rock(&mut self) {
        if let Some((rock, loc)) = self.falling_rock.take() {
            let nloc = Loc(loc.0, loc.1 + 1);
            for y in 0..rock.h {
                for x in 0..rock.w {
                    let iloc = Loc(x as isize, y as isize);
                    if nloc.1 + rock.h as isize > 0
                        || (rock.blocked(iloc) && self.blocked(nloc + iloc))
                    {
                        let top = loc.1 - rock.h as isize;
                        let newmax = (top.neg() * CHAMBER_WIDTH) as usize;
                        // fill
                        self.occupancy
                            .resize(self.occupancy.len().max(newmax), false);
                        for iy in 0..rock.h {
                            for ix in 0..rock.w {
                                let riloc = Loc(ix as isize, iy as isize);
                                if rock.blocked(riloc) {
                                    self.block(loc + riloc);
                                }
                            }
                        }
                        self.falling_rock = None;
                        return;
                    }
                }
            }
            self.falling_rock = Some((rock, nloc));
        }
    }
    fn stack_height(&self) -> isize {
        let my = (self.occupancy.len() as isize / CHAMBER_WIDTH).neg();
        for y in my..0 {
            for x in 0..CHAMBER_WIDTH {
                if self.blocked(Loc(x, y)) {
                    return y.neg();
                }
            }
        }
        0
    }
    fn blocked(&self, loc: Loc) -> bool {
        let rawloc = -loc.1 * CHAMBER_WIDTH + loc.0;
        assert!(!rawloc.is_negative());
        let rawloc = rawloc as usize;
        if self.occupancy.len() > rawloc {
            self.occupancy[rawloc]
        } else {
            false
        }
    }
    fn block(&mut self, loc: Loc) {
        let rawloc = -loc.1 * CHAMBER_WIDTH + loc.0;
        assert!(!rawloc.is_negative());
        let rawloc = rawloc as usize;
        if self.occupancy.len() > rawloc {
            self.occupancy[rawloc] = true;
        } else {
            panic!("oob");
        }
    }
    fn floordepth(&self) -> [usize; CHAMBER_WIDTH as usize] {
        let mut out = [0; CHAMBER_WIDTH as usize];
        let sh = self.stack_height().neg();
        for x in 0..CHAMBER_WIDTH {
            for y in sh..=0 {
                if self.blocked(Loc(x, y)) || y == 0 {
                    out[x as usize] = (y - sh) as usize;
                    break;
                }
            }
        }
        out
    }
}

impl Display for Chamber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let top = -self.stack_height() - 8;
        for y in top..=0 {
            for x in -1..=CHAMBER_WIDTH {
                let tloc = Loc(x, y);
                if y == 0 && (x == -1 || x == CHAMBER_WIDTH) {
                    f.write_char('+')?;
                } else if y == 0 {
                    f.write_char('-')?;
                } else if x == -1 || x == CHAMBER_WIDTH {
                    f.write_char('|')?;
                } else {
                    let mut ch = '.';
                    if let Some((rock, loc)) = &self.falling_rock {
                        let rloc = tloc - *loc;
                        if rloc.1 >= 0
                            && rloc.0 >= 0
                            && rloc.0 < rock.w as isize
                            && rloc.1 < rock.h as isize
                            && rock.blocked(rloc)
                        {
                            ch = '@';
                        }
                    }
                    if self.blocked(tloc) {
                        ch = '#';
                    }
                    f.write_char(ch)?;
                }
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let input = io::read_to_string(io::stdin())?;
    let pseq: Vec<Push> = input
        .trim()
        .chars()
        .map(|c| match c {
            '<' => Push::Left,
            '>' => Push::Right,
            c => panic!("unexpected input char '{}'", c),
        })
        .collect();

    let rocks: Vec<Rock> = vec![
        "####",
        ".#.
         ###
         .#.",
        "..#
         ..#
         ###",
        "#
         #
         #
         #",
        "##
         ##",
    ]
    .into_iter()
    .map(Rock::from)
    .collect();

    let mut chamber = Chamber::default();
    chamber.spawn_rock(rocks[0].clone());
    let mut ridx = 0;
    let mut pidx = 0;
    let mut stopped: usize = 0;
    println!("{chamber}");
    let mut lsh = chamber.stack_height();
    let mut steps = HashSet::new();
    let mut ccount = false;
    let mut cinc: usize = 0;
    let mut clen = 0;
    let mut cycle = None;
    let mut bh = 0;
    loop {
        chamber.push_rock(pseq[pidx]);
        pidx = (pidx + 1) % pseq.len();
        //println!("{chamber}");
        chamber.drop_rock();
        //println!("{chamber}");
        if chamber.falling_rock.is_none() {
            stopped += 1;
            let sh = chamber.stack_height();
            let linc = sh - lsh;
            lsh = sh;
            if ccount {
                cinc += linc as usize;
                clen += 1;
            }
            let fd = chamber.floordepth();
            if cycle.is_none() {
                let ck = (fd, pidx, ridx);
                if !steps.insert(ck) {
                    println!("Found cycle beginning with {ck:?}");
                    if ccount {
                        println!("Cycle height/len: {cinc} {clen}");
                        cycle = Some((cinc, clen));
                        let sl = 1000000000000 - stopped ;
                        let ccount = sl / clen;
                        stopped  += ccount * clen;
                        bh = ccount * cinc;
                        println!("bump height: {}", bh);
                    } else {
                        steps.clear();
                        steps.insert(ck);
                        ccount = true;
                    }
                };
            }
            if stopped == 2022 || stopped == 1000000000000 {
                println!(
                    "{stopped} rocks stopped, stack height {sh}, last increase {linc}"
                );
                println!("bump height {} total height {}", bh, sh as usize + bh);
                if stopped == 1000000000000 {
                    break;
                }
            }
            ridx = (ridx + 1) % rocks.len();
            chamber.spawn_rock(rocks[ridx].clone());
        }
    }
    Ok(())
}
