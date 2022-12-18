use color_eyre::Result;
use std::{
    collections::{hash_map, HashMap, HashSet},
    io,
    ops::Add, hash::Hash,
};

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

    let mins = cubelocs
        .iter()
        .copied()
        .reduce(|a, b| Loc3(a.0.min(b.0), a.1.min(b.1), a.2.min(b.2)))
        .unwrap();
    let maxs = cubelocs
        .iter()
        .copied()
        .reduce(|a, b| Loc3(a.0.max(b.0), a.1.max(b.1), a.2.max(b.2)))
        .unwrap();

    let oob = |loc: Loc3| {
        loc.0 < mins.0
            || loc.1 < mins.1
            || loc.2 < mins.2
            || loc.0 > maxs.0
            || loc.1 > maxs.1
            || loc.2 > maxs.2
    };

    let mut exterior_surface = 0;
    let mut oob_reachable = cached(|loc: Loc3| {
        pathfinding::prelude::bfs(
            &loc,
            |l| l.adjacents().filter(|a| !cubeset.contains(a)),
            |l| oob(*l),
        )
        .is_some()
    });
    for cube in cubeset.iter() {
        for adj in cube.adjacents() {
            if !cubeset.contains(&adj) && oob_reachable(adj) {
                exterior_surface += 1;
            }
        }
    }
    println!("P2 exterior surface sides: {}", exterior_surface);

    Ok(())
}

fn cached<I, O>(f: impl Fn(I) -> O) -> impl FnMut(I) -> O
where
    I: Hash + Eq + Copy,
    O: Copy,
{
    let mut cache: HashMap<I, O> = Default::default();
    use hash_map::Entry::*;
    move |k: I| match cache.entry(k) {
        Vacant(e) => {
            let computed = f(k);
            e.insert(computed);
            computed
        }
        Occupied(v) => *v.get(),
    }
}
