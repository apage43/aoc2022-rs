use color_eyre::Result;
fn main() -> Result<()> {
    color_eyre::install()?;
    let mut inventories = Vec::new();
    let mut curinv: Option<Vec<u32>> = None;
    for line in std::io::stdin().lines() {
        let line = line?;
        if line.is_empty() {
            if let Some(completed) = curinv.take() {
                inventories.push(completed);
            }
            continue;
        }
        let calories: u32 = line.parse()?;
        curinv.get_or_insert_with(Vec::new).push(calories);
    }
    if let Some(completed) = curinv.take() {
        inventories.push(completed);
    }

    eprintln!("Inventories: {:#?}", inventories);
    let mut sums: Vec<u32> = inventories.iter().map(|x| x.iter().sum()).collect();
    sums.sort();
    eprintln!("Sums: {:#?}", sums);
    let maxsum = sums.last();
    if let Some(maxsum) = maxsum {
        println!("max sum: {}", maxsum);
    }
    let top3sums = &sums[sums.len() - 3..];
    let top3total: u32 = top3sums.iter().sum();
    println!("top3 total: {}", top3total);

    Ok(())
}
