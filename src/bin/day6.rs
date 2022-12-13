use color_eyre::Result;

fn sop_index(line: &str, n_distinct: usize) -> Option<usize> {
    if line.len() < n_distinct {
        return None;
    }
    for i in n_distinct..line.len() {
        let marker = &line[i - n_distinct..i];
        // assuming these are all lowercase ascii so we can just use a u32 and popcnt
        let mut mask: u32 = 0;
        for b in marker.as_bytes() {
            let letnum = b - b'a';
            assert!(letnum < 32);
            mask |= 1 << letnum;
        }
        if mask.count_ones() == n_distinct as u32 {
            return Some(i);
        }
    }
    None
}

fn main() -> Result<()> {
    color_eyre::install()?;
    for line in std::io::stdin().lines() {
        let line = line?;
        // part1
        println!("Start of packet: {:?}", sop_index(&line, 4));
        // part2
        println!("Start of msg: {:?}", sop_index(&line, 14));
    }
    Ok(())
}
