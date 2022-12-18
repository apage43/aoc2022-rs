use color_eyre::Result;
use std::{collections::HashSet, io, ops::Add};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct Loc3(isize, isize, isize);

impl Add for Loc3 {
    type Output = Loc3;

    fn add(self, rhs: Self) -> Self::Output {
        Loc3(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}

impl Loc3 {
    fn adjacents(self) -> impl Iterator<Item = Loc3> {
        let sides = [
            Loc3(1, 0, 0),
            Loc3(0, 1, 0),
            Loc3(0, 0, 1),
            Loc3(-1, 0, 0),
            Loc3(0, -1, 0),
            Loc3(0, 0, -1),
        ];
        sides.into_iter().map(move |s| self + s)
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let input = io::read_to_string(io::stdin())?;
    let cubelocs: Vec<Loc3> = input
        .lines()
        .map(|l| {
            let mut xyz = l.splitn(3, ',');
            Ok(Loc3(
                xyz.next().expect("expected comma").parse()?,
                xyz.next().expect("expected comma").parse()?,
                xyz.next().expect("expected comma").parse()?,
            ))
        })
        .collect::<Result<_>>()?;
    let cubeset: HashSet<Loc3> = HashSet::from_iter(cubelocs.iter().copied());
    let mut unconnected_sides = 0;
    for cube in cubeset.iter() {
        for adj in cube.adjacents() {
            if !cubeset.contains(&adj) {
                unconnected_sides += 1;
            }
        }
    }
    println!("P1 unconnected sides: {unconnected_sides}");

    Ok(())
}
