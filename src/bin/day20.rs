use color_eyre::Result;
use std::io;

fn do_moves(original: &Vec<(usize, i64)>, targ: &mut Vec<(usize, i64)>) {
    for mover in original {
        let pos = targ
            .iter()
            .position(|&n| n == *mover)
            .expect("cant find number");
        assert!(targ.remove(pos) == *mover);
        let mut npos = (pos as isize + mover.1 as isize) % targ.len() as isize;
        if npos <= 0 {
            npos += targ.len() as isize;
        }
        targ.insert(npos as usize, *mover);
        if targ.len() < 100 {
            //example:
            println!("{:?}", targ);
        }
    }
}

fn coord(moved: &[(usize, i64)]) -> i64 {
    let zeropos = moved.iter().position(|&n| n.1 == 0).expect("no zero");
    [zeropos + 1000, zeropos + 2000, zeropos + 3000]
        .into_iter()
        .map(|i| i % moved.len())
        .map(|i| moved[i].1)
        .sum()
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let input = io::read_to_string(io::stdin())?;
    let original: Vec<(usize, i64)> = input
        .lines()
        .map(|l| l.parse().unwrap())
        .enumerate()
        .collect();
    let mut p1moved = original.clone();
    if p1moved.len() < 100 {
        //example:
        println!("{:?}", p1moved);
    }
    do_moves(&original, &mut p1moved);
    println!("p1sum: {}", coord(&p1moved));

    let p2o: Vec<(usize, i64)> = original
        .into_iter()
        .map(|(i, n)| (i, n * 811589153))
        .collect();
    let mut p2moved = p2o.clone();
    for _ in 0..10 {
        do_moves(&p2o, &mut p2moved);
    }
    println!("p2sum: {}", coord(&p2moved));

    Ok(())
}
