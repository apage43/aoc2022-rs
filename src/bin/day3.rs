use color_eyre::{eyre::{bail, ContextCompat}, Result};
use std::{collections::HashSet, hash::Hash};

#[derive(Eq, PartialEq, Copy, Clone, Hash, Debug)]
struct Item(char);

impl TryFrom<char> for Item {
    type Error = color_eyre::Report;
    fn try_from(other: char) -> Result<Item> {
        if other.is_ascii_alphabetic() {
            Ok(Item(other))
        } else {
            bail!("invalid item")
        }
    }
}

impl Item {
    fn priority(self) -> u32 {
        if self.0.is_lowercase() {
            1 + self.0 as u32 - 'a' as u32
        } else {
            27 + self.0 as u32 - 'A' as u32
        }
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut missort_total_prio = 0;
    let mut total_badge_prio = 0;
    let mut current_group = Vec::new();
    for line in std::io::stdin().lines() {
        let line = line?;
        let items: Vec<Item> = line.chars().map(Item::try_from).collect::<Result<_>>()?;
        let (part_a, part_b) = items.split_at(items.len() / 2);
        if part_a.len() != part_b.len() {
            bail!("uneven rucksack!");
        }
        let paset: HashSet<Item> = part_a.iter().cloned().collect();
        let pbset: HashSet<Item> = part_b.iter().cloned().collect();
        let missort = paset.intersection(&pbset).next().context("no missort?")?;
        let msprio = missort.priority();
        missort_total_prio += msprio;
        eprintln!("missort: {:?}, {}", missort, msprio);
        current_group.push(items);
        if current_group.len() == 3 {
            let sets: Vec<HashSet<_>> = current_group
                .iter()
                .map(|g| g.iter().cloned().collect::<HashSet<_>>())
                .collect();
            let g1set: HashSet<_> = sets[0].intersection(&sets[1]).cloned().collect();
            let badge = g1set.intersection(&sets[2]).next().context("no badge?")?;
            let badge_prio = badge.priority();
            current_group.clear();
            total_badge_prio += badge_prio;
            eprintln!("badge: {:?} ({})", badge, badge_prio);
        }
    }
    println!("Total missort priority: {}", missort_total_prio);
    println!("Total badge priority: {}", total_badge_prio);
    Ok(())
}
