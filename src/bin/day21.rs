use color_eyre::Result;
use regex::Regex;
use std::{collections::HashMap, hash::Hash, io};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct MonkeyId(usize);

#[derive(Default)]
struct MonkeyMap {
    name_to_id: HashMap<String, MonkeyId>,
    id_to_name: HashMap<MonkeyId, String>,
    next_id: usize,
}

impl MonkeyMap {
    fn get_or_insert(&mut self, mkname: &str) -> MonkeyId {
        if let Some(existing) = self.name_to_id.get(mkname) {
            *existing
        } else {
            let id = MonkeyId(self.next_id);
            let ok = mkname.to_owned();
            self.name_to_id.insert(ok.clone(), id);
            self.id_to_name.insert(id, ok);
            self.next_id += 1;
            id
        }
    }
}

enum MonkeyRule {
    Shout(i64),
    Add(MonkeyId, MonkeyId),
    Sub(MonkeyId, MonkeyId),
    Mul(MonkeyId, MonkeyId),
    Div(MonkeyId, MonkeyId),
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let input = io::read_to_string(io::stdin())?;
    let mmre = Regex::new(r"(\w+): ((\d+)|((\w+) (.) (\w+)))").unwrap();
    let mut mmap = MonkeyMap::default();
    let mut mrules: HashMap<MonkeyId, MonkeyRule> = Default::default();
    for line in input.lines() {
        let mlcap = mmre.captures(line).expect("monkey line");
        let mname = &mlcap[1];
        //println!("mn={mname}: {mlcap:?}");
        let mid = mmap.get_or_insert(mname);
        let mrule = if let Some(shout) = mlcap.get(3) {
            MonkeyRule::Shout(shout.as_str().parse()?)
        } else if let Some(lhs) = mlcap.get(5) {
            let lhs = mmap.get_or_insert(lhs.as_str());
            let rhs = mmap.get_or_insert(mlcap.get(7).expect("expr rhs").as_str());
            let op = mlcap.get(6).expect("operator").as_str();
            match op {
                "+" => MonkeyRule::Add(lhs, rhs),
                "-" => MonkeyRule::Sub(lhs, rhs),
                "*" => MonkeyRule::Mul(lhs, rhs),
                "/" => MonkeyRule::Div(lhs, rhs),
                _ => panic!("bad operator: {}", op),
            }
        } else {
            panic!("bad monkey line: {}", line)
        };
        mrules.insert(mid, mrule);
    }
    let rootid = mmap.get_or_insert("root");
    let mut results: HashMap<MonkeyId, i64> = Default::default();
    while results.values().len() < mrules.len() {
        let incompletes: Vec<MonkeyId> = mrules
            .keys()
            .filter(|p| results.get(*p).is_none())
            .copied()
            .collect();
        for mid in incompletes {
            if let Some(result) = (|| {
                use MonkeyRule::*;
                let rule = mrules.get(&mid).expect("no rule for monkey");
            
                match &rule {
                    Shout(v) => Some(*v),
                    Add(r,l) => Some(results.get(r)? + results.get(l)?),
                    Sub(r,l) => Some(results.get(r)? - results.get(l)?),
                    Mul(r,l) => Some(results.get(r)? * results.get(l)?),
                    Div(r,l) => Some(results.get(r)? / results.get(l)?),
                }
            })() {
                results.insert(mid, result);
            }
        }
    }
    let rootresult = results.get(&rootid);
    println!("p1 root monkey result: {rootresult:?}");

    Ok(())
}
