use color_eyre::{
    eyre::{ContextCompat},
    Result,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Interval {
    start: u32,
    end: u32,
}

impl Interval {
    // self fully contains other
    fn contains(&self, other: &Interval) -> bool {
        self.start <= other.start && self.end >= other.end
    }

    // self overlaps other at all
    fn overlaps(&self, other: &Interval) -> bool {
        self.start.max(other.start) <= self.end.min(other.end)
    }
}
fn main() -> Result<()> {
    color_eyre::install()?;
    let mut total_full_overlaps = 0;
    let mut total_partial_overlaps = 0;
    for line in std::io::stdin().lines() {
        let line = line?;
        let (a, b, c, d) = strp::try_scan!(line => "{}-{},{}-{}")
            .ok()
            .context("parse interval pair")?;
        let ia = Interval { start: a, end: b };
        let ib = Interval { start: c, end: d };
        if ia.contains(&ib) || ib.contains(&ia) { total_full_overlaps += 1; }
        if ia.overlaps(&ib) { total_partial_overlaps += 1; }
    }
    println!("Total full overlaps: {}", total_full_overlaps);
    println!("Total partial overlaps: {}", total_partial_overlaps);
    Ok(())
}
