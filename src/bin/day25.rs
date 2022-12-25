use color_eyre::Result;
use std::io;

fn sn2digit(c: char) -> i64 {
    match c {
        '2' => 2,
        '1' => 1,
        '0' => 0,
        '-' => -1,
        '=' => -2,
        _ => unimplemented!(),
    }
}

fn tosn(n: i64) -> String {
    let mut n = n;
    let mut out = Vec::new();
    while n != 0 {
        let digit = n % 5;
        out.push(match digit {
            0 => '0',
            1 => '1',
            2 => '2',
            3 => {n+=2; '='},
            4 => {n+=1; '-'},
            _ => unimplemented!()
        });
        n /= 5
    }
    out.into_iter().rev().collect()
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let input = io::read_to_string(io::stdin())?;
    let mut total = 0;
    for line in input.lines() {
        let val: i64 = std::iter::zip(line.chars().map(sn2digit), (0..line.len()).rev())
            .map(|(v, place)| v * (5_i64.pow(place as u32)))
            .sum();
        println!("{line} = {val}");
        total += val;
    }
    println!("total: {total}");
    println!("ans: {}", tosn(total));
    Ok(())
}
