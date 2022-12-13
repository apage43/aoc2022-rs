use std::collections::{BTreeMap, HashMap};

use color_eyre::{eyre::ContextCompat, Result};

#[derive(Debug, Eq, PartialEq)]
enum DirectoryEntry {
    Directory,
    File(u64),
}

fn main() -> Result<()> {
    color_eyre::install()?;
    // Ord for Vec compares each element in order, so representing paths as a Vec<String>
    // in an ordered map both makes for an easy way both to deal with cwd, and makes each dir's
    // recursive subelements lie in a contiguous range we can easily iterate over.
    let mut filesystem: BTreeMap<Vec<String>, DirectoryEntry> = BTreeMap::new();
    let mut reading_ls = false;
    let mut cwd = Vec::new();
    filesystem.insert(Vec::new(), DirectoryEntry::Directory);
    for line in std::io::stdin().lines() {
        let line = line?;
        if line.starts_with('$') {
            reading_ls = false;
        }
        if let Some(mut target) = line.strip_prefix("$ cd ") {
            if target.starts_with('/') {
                cwd.clear();
                target = &target[1..];
            }
            if target.is_empty() {
                continue;
            }
            for pel in target.split('/') {
                if pel == ".." {
                    cwd.pop();
                } else {
                    cwd.push(pel.to_owned());
                }
            }
        }
        if line.starts_with("$ ls") {
            reading_ls = true;
            continue;
        }
        if !line.starts_with('$') && reading_ls {
            if let Some(dirname) = line.strip_prefix("dir ") {
                let mut dirpath = cwd.clone();
                dirpath.push(dirname.to_owned());
                filesystem.insert(dirpath, DirectoryEntry::Directory);
            } else {
                let (sizestr, filename) = line.split_once(' ').context("no space on ls line?")?;
                let fsize: u64 = sizestr.parse()?;
                let mut filepath = cwd.clone();
                filepath.push(filename.to_owned());
                filesystem.insert(filepath, DirectoryEntry::File(fsize));
            }
        }
    }
    let dirs: Vec<Vec<String>> = filesystem
        .iter()
        .filter(|(_k, v)| **v == DirectoryEntry::Directory)
        .map(|(k, _v)| k.to_owned())
        .collect();
    let mut dirsizes: HashMap<_, u64> = HashMap::new();
    for dir in dirs {
        let mut dsize = 0;
        let entries = filesystem.range(dir.clone()..);
        for (path, dirent) in entries {
            if !path.starts_with(&dir) {
                break;
            }
            if let DirectoryEntry::File(fsize) = dirent {
                dsize += fsize;
            }
        }
        dirsizes.insert(dir, dsize);
    }
    eprintln!("Dirsizes: {:#?}", dirsizes);
    // part1
    let totalover100k: u64 = dirsizes.values().filter(|s| **s <= 100000).sum();
    println!("Part1, sum of dirs <=100k: {}", totalover100k);
    // part2
    const TOTAL_SPACE: u64 = 70000000;
    const NEEDED_SPACE: u64 = 30000000;
    let used_space = *dirsizes.get(&vec![]).context("size of /")?;
    let free_space = TOTAL_SPACE - used_space;
    let needed_additional_space = NEEDED_SPACE - free_space;
    let big_enough = dirsizes.values().filter(|s| **s >= needed_additional_space);
    let min_bigenough = *big_enough.min().context("big enough?")?;
    println!("Part2, delete a dir of size: {}", min_bigenough);
    Ok(())
}
