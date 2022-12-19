use color_eyre::Result;
use rayon::prelude::*;
use regex::Regex;
use std::{collections::HashSet, hash::Hash, io};

#[derive(Debug, Clone, Copy)]
struct Blueprint {
    bpid: u16,
    ore_ore_cost: u16,
    clay_ore_cost: u16,
    obsidian_ore_cost: u16,
    obsidian_clay_cost: u16,
    geode_ore_cost: u16,
    geode_obsidian_cost: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Material {
    Ore,
    Clay,
    Obsidian,
    Geode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Action {
    Wait,
    BuildRobot(Material),
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
struct State {
    time_elapsed: u8,
    ore_bots: u16,
    ore_held: u16,
    clay_bots: u16,
    clay_held: u16,
    obsidian_bots: u16,
    obsidian_held: u16,
    geode_bots: u16,
    geode_held: u16,
}

impl State {
    fn new() -> State {
        State {
            time_elapsed: 0,
            ore_bots: 1,
            ore_held: 0,
            clay_bots: 0,
            clay_held: 0,
            obsidian_bots: 0,
            obsidian_held: 0,
            geode_bots: 0,
            geode_held: 0,
        }
    }
    fn do_production(&mut self) {
        self.ore_held += self.ore_bots;
        self.clay_held += self.clay_bots;
        self.obsidian_held += self.obsidian_bots;
        self.geode_held += self.geode_bots;
    }
    fn actions_possible(&self, blueprint: &Blueprint) -> impl Iterator<Item = Action> {
        use Action::*;
        use Material::*;
        let geode_affordable = self.ore_held >= blueprint.geode_ore_cost
            && self.obsidian_held >= blueprint.geode_obsidian_cost;
        let obsidian_affordable = self.ore_held >= blueprint.obsidian_ore_cost
            && self.clay_held >= blueprint.obsidian_clay_cost;
        let clay_affordable = self.ore_held >= blueprint.clay_ore_cost;
        let ore_affordable = self.ore_held >= blueprint.ore_ore_cost;
        [
            (!geode_affordable).then_some(Wait),
            (ore_affordable && !geode_affordable).then_some(BuildRobot(Ore)),
            (clay_affordable && !geode_affordable).then_some(BuildRobot(Clay)),
            (obsidian_affordable && !geode_affordable).then_some(BuildRobot(Obsidian)),
            (geode_affordable).then_some(BuildRobot(Geode)),
        ]
        .into_iter()
        .flatten()
    }
    fn do_action(&self, action: Action, blueprint: &Blueprint) -> State {
        let mut new = *self;
        new.do_production();
        use Action::*;
        use Material::*;
        match action {
            BuildRobot(Ore) => {
                new.ore_bots += 1;
                new.ore_held -= blueprint.ore_ore_cost;
            }
            BuildRobot(Clay) => {
                new.clay_bots += 1;
                new.ore_held -= blueprint.clay_ore_cost;
            }
            BuildRobot(Obsidian) => {
                new.obsidian_bots += 1;
                new.ore_held -= blueprint.obsidian_ore_cost;
                new.clay_held -= blueprint.obsidian_clay_cost;
            }
            BuildRobot(Geode) => {
                new.geode_bots += 1;
                new.ore_held -= blueprint.geode_ore_cost;
                new.obsidian_held -= blueprint.geode_obsidian_cost;
            }
            Wait => {}
        };
        new.time_elapsed += 1;
        new
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let bpre = Regex::new(r"Blueprint (\d+): (.+)")?;
    let rulere = Regex::new(r"Each (\w+) robot costs (\d+) (\w+)( and (\d+) (\w+))?.\s*")?;
    let input = io::read_to_string(io::stdin())?;
    let mut blueprints = vec![];
    for line in input.lines() {
        let bpidcap = bpre.captures(line).expect("Bad bp line");
        let bpid = bpidcap[1].parse()?;
        let mut rules = rulere.captures_iter(&bpidcap[2]);
        let orecap = rules.next().expect("ore rule");
        assert!(orecap[1] == *"ore");
        let ore_ore_cost = orecap[2].parse()?;
        let claycap = rules.next().expect("clay rule");
        assert!(claycap[1] == *"clay");
        let clay_ore_cost = claycap[2].parse()?;
        let obscap = rules.next().expect("obsidian rule");
        assert!(obscap[1] == *"obsidian");
        let obsidian_ore_cost = obscap[2].parse()?;
        let obsidian_clay_cost = obscap[5].parse()?;
        let geocap = rules.next().expect("geode rule");
        let geode_ore_cost = geocap[2].parse()?;
        let geode_obsidian_cost = geocap[5].parse()?;
        blueprints.push(Blueprint {
            bpid,
            ore_ore_cost,
            clay_ore_cost,
            obsidian_ore_cost,
            obsidian_clay_cost,
            geode_ore_cost,
            geode_obsidian_cost,
        });
    }
    println!("Checking {} blueprints...", blueprints.len());
    let total_qlv: usize = blueprints
        .clone()
        .into_par_iter()
        .map(|bp| {
            let geodes = score_blueprint(bp, 24);
            let qlv = geodes * bp.bpid;
            println!("bp{} 24min score={geodes}, qlv={}", bp.bpid, qlv);
            qlv as usize
        })
        .sum();
    println!("Total qlv: {total_qlv}\n");

    let p2ans: usize = blueprints
        .into_par_iter()
        .take(3)
        .map(|bp| {
            let geodes = score_blueprint(bp, 32);
            println!("bp{} 32min score={geodes}", bp.bpid);
            geodes as usize
        })
        .product();
    println!("Top3 product: {p2ans}");
    Ok(())
}

fn score_blueprint(bp: Blueprint, time_limit: u8) -> u16 {
    let mut states = vec![State::new()];
    let mut best_state: Option<State> = None;
    let mut seen = HashSet::new();
    while let Some(state) = states.pop() {
        if seen.contains(&state) {
            continue;
        } else {
            seen.insert(state);
        }
        let geode_best = best_state.map(|s| s.geode_held).unwrap_or(0);
        let time_left = time_limit - state.time_elapsed;

        // could we beat the current best if geode bots were free?
        let geode_naive_upper_bound = state.geode_held
            + (0..time_left)
                .map(|n| state.geode_bots + n as u16)
                .sum::<u16>();
        if geode_naive_upper_bound < geode_best {
            continue;
        }

        if state.time_elapsed < time_limit {
            for nact in state.actions_possible(&bp) {
                states.push(state.do_action(nact, &bp));
            }
        } else if let Some(best) = best_state {
            if state.geode_held > best.geode_held {
                best_state = Some(state)
            }
        } else {
            best_state = Some(state)
        }
    }
    best_state.map(|s| s.geode_held).unwrap_or(0)
}
