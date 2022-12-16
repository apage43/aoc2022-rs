use color_eyre::{eyre::eyre, Result};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    io,
};

#[derive(Debug, Clone)]
struct Valve {
    flowrate: u32,
    exits: Vec<String>,
}

#[derive(Clone, Debug)]
struct State<'a> {
    time_elapsed: u32,
    pressure_released: u32,
    open_valves: rpds::HashTrieSet<&'a str>,
    at_valve: &'a str,
    seeking: Option<&'a str>,
    history: rpds::List<Action<'a>>,
}

fn path_toward<'a>(from: &'a str, to: &'a str, valvemap: &'a HashMap<String, Valve>) -> &'a str {
    let mut sq = VecDeque::new();
    sq.push_back((from, None));
    let mut visited = HashSet::new();
    visited.insert(from);
    // eprintln!("Search path {} -> {}", from, to);
    while let Some((n, step)) = sq.pop_front() {
        if n == to {
            return step.expect("from==to");
        }
        visited.insert(n);
        for next in valvemap.get(n).unwrap().exits.iter() {
            if visited.contains(next.as_str()) {
                continue;
            }
            sq.push_back((next, step.or(Some(next))))
        }
    }
    panic!("no path found");
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Action<'a> {
    Open(&'a str),
    MoveToward(&'a str),
    Wait,
}
impl<'a> State<'a> {
    fn actions(&self, valvemap: &'a HashMap<String, Valve>) -> Vec<Action<'a>> {
        let mut actions = vec![];

        if self.time_elapsed > 30 {
            return actions;
        }

        if let Some(seek) = self.seeking {
            if self.at_valve != seek {
                actions.push(Action::MoveToward(seek));
                return actions;
            } else {
                actions.push(Action::Open(self.at_valve));
            }
        }

        let valves_left =
            self.open_valves.size() < valvemap.values().filter(|v| v.flowrate > 0).count();
        if valves_left {
            let remain = valvemap
                .iter()
                .filter(|(k, v)| v.flowrate > 0 && !self.open_valves.contains(k.as_str()));
            for (vn, _valve) in remain {
                if self.at_valve != vn {
                    actions.push(Action::MoveToward(vn.as_str()));
                }
            }
        } else {
            actions.push(Action::Wait);
        }

        actions
    }
    fn pressure_tick(&self, valvemap: &'a HashMap<String, Valve>) -> u32 {
        self.open_valves
            .iter()
            .map(|v| valvemap.get(*v).expect("bad valve name").flowrate)
            .sum::<u32>()
    }
    fn apply_action(self, valvemap: &'a HashMap<String, Valve>, action: Action<'a>) -> State<'a> {
        match action {
            Action::Wait => State {
                time_elapsed: self.time_elapsed + 1,
                pressure_released: self.pressure_released + self.pressure_tick(valvemap),
                open_valves: self.open_valves.clone(),
                at_valve: self.at_valve,
                seeking: None,
                history: self.history.push_front(action),
            },
            Action::Open(_) => State {
                time_elapsed: self.time_elapsed + 1,
                pressure_released: self.pressure_released + self.pressure_tick(valvemap),
                open_valves: self.open_valves.insert(self.at_valve),
                at_valve: self.at_valve,
                seeking: None,
                history: self.history.push_front(action),
            },
            Action::MoveToward(dest) => State {
                time_elapsed: self.time_elapsed + 1,
                pressure_released: self.pressure_released + self.pressure_tick(valvemap),
                open_valves: self.open_valves.clone(),
                at_valve: path_toward(self.at_valve, dest, valvemap),
                seeking: Some(dest),
                history: self.history.push_front(action),
            },
        }
    }
}

fn do_part1(valvemap: &HashMap<String, Valve>) -> Result<()> {
    let mut expstack = Vec::new();
    expstack.push(State {
        time_elapsed: 0,
        pressure_released: 0,
        open_valves: Default::default(),
        at_valve: "AA",
        seeking: None,
        history: Default::default(),
    });
    let mut best_rate: HashMap<u32, u32> = HashMap::new();
    let mut best_pressure = 0;
    let mut best_state = None;
    while let Some(curstate) = expstack.pop() {
        {
            let bpe = best_rate.entry(curstate.time_elapsed).or_default();
            if curstate.pressure_released > *bpe {
                *bpe = curstate.pressure_released;
            }
            if curstate.pressure_released
                < *best_rate
                    .get(&curstate.time_elapsed.saturating_sub(1))
                    .unwrap_or(&0)
            {
                continue;
            }
        }
        if curstate.time_elapsed <= 30 && curstate.pressure_released > best_pressure {
            best_pressure = curstate.pressure_released;
            best_state = Some(curstate.clone())
        }
        let actions = curstate.actions(valvemap);
        for action in actions {
            expstack.push(curstate.clone().apply_action(valvemap, action));
        }
    }
    println!("Part 1 best pressure: {:?}", best_pressure);
    if let Some(best_state) = best_state {
        let history = best_state.history.reverse();
        let acts: Vec<&Action<'_>> = history.iter().collect();
        println!("{:?}", acts);
    }

    Ok(())
}

#[derive(Clone, Debug)]
struct StateWithElephant<'a> {
    time_elapsed: u32,
    pressure_released: u32,
    open_valves: rpds::HashTrieSet<&'a str>,
    my_valve: &'a str,
    elephant_valve: &'a str,
    my_seeking: Option<&'a str>,
    elephant_seeking: Option<&'a str>,
    history: rpds::List<(Action<'a>, Action<'a>)>,
}
impl<'a> StateWithElephant<'a> {
    fn actions(&self, valvemap: &'a HashMap<String, Valve>) -> Vec<(Action<'a>, Action<'a>)> {
        let mut my_actions: Vec<Action<'a>> = vec![];
        let mut elephant_actions: Vec<Action<'a>> = vec![];

        if self.time_elapsed > 26 {
            return vec![];
        }

        let valves_left =
            self.open_valves.size() < valvemap.values().filter(|v| v.flowrate > 0).count();

        if let Some(meseek) = self.my_seeking {
            if self.my_valve != meseek {
                my_actions.push(Action::MoveToward(meseek));
            } else {
                my_actions.push(Action::Open(self.my_valve));
            }
        } else if valves_left {
            let remain = valvemap
                .iter()
                .filter(|(k, v)| v.flowrate > 0 && !self.open_valves.contains(k.as_str()));
            for (vn, _valve) in remain {
                if self.my_valve != vn {
                    my_actions.push(Action::MoveToward(vn.as_str()));
                }
            }
        } else {
            my_actions.push(Action::Wait);
        }

        if let Some(elephantseek) = self.elephant_seeking {
            if self.elephant_valve != elephantseek {
                elephant_actions.push(Action::MoveToward(elephantseek));
            } else {
                elephant_actions.push(Action::Open(self.elephant_valve));
            }
        } else if valves_left {
            let remain = valvemap
                .iter()
                .filter(|(k, v)| v.flowrate > 0 && !self.open_valves.contains(k.as_str()));
            for (vn, _valve) in remain {
                if self.elephant_valve != vn {
                    elephant_actions.push(Action::MoveToward(vn.as_str()));
                }
            }
        } else {
            elephant_actions.push(Action::Wait);
        }

        let mut product = vec![];
        for ma in my_actions {
            for ea in elephant_actions.iter().copied() {
                // don't do the same thing as the elephant unless it is waiting.
                if ma == Action::Wait || ma != ea {
                    product.push((ma, ea));
                }
            }
        }
        product
    }
    fn pressure_tick(&self, valvemap: &'a HashMap<String, Valve>) -> u32 {
        self.open_valves
            .iter()
            .map(|v| valvemap.get(*v).expect("bad valve name").flowrate)
            .sum::<u32>()
    }
    fn apply_actions(
        self,
        valvemap: &'a HashMap<String, Valve>,
        my_action: Action<'a>,
        elephant_action: Action<'a>,
    ) -> StateWithElephant<'a> {
        let mut open_valves = self.open_valves.clone();
        let (my_valve, my_seeking) = match my_action {
            Action::Wait => (self.my_valve, None),
            Action::Open(at) => {
                open_valves = open_valves.insert(at);
                (self.my_valve, None)
            }
            Action::MoveToward(v) => (path_toward(self.my_valve, v, valvemap), Some(v)),
        };
        let (elephant_valve, elephant_seeking) = match elephant_action {
            Action::Wait => (self.elephant_valve, None),
            Action::Open(at) => {
                open_valves = open_valves.insert(at);
                (self.elephant_valve, None)
            }
            Action::MoveToward(v) => (path_toward(self.elephant_valve, v, valvemap), Some(v)),
        };
        StateWithElephant {
            time_elapsed: self.time_elapsed + 1,
            pressure_released: self.pressure_released + self.pressure_tick(valvemap),
            open_valves,
            my_valve,
            elephant_valve,
            my_seeking,
            elephant_seeking,
            history: self.history.push_front((my_action, elephant_action)),
        }
    }
}

fn do_part2(valvemap: &HashMap<String, Valve>) -> Result<()> {
    let mut expstack = Vec::new();
    expstack.push(StateWithElephant {
        time_elapsed: 0,
        pressure_released: 0,
        open_valves: Default::default(),
        my_valve: "AA",
        my_seeking: None,
        elephant_valve: "AA",
        elephant_seeking: None,
        history: Default::default(),
    });
    let mut best_rate: HashMap<u32, u32> = HashMap::new();
    let mut best_pressure = 0;
    let mut best_state = None;
    while let Some(curstate) = expstack.pop() {
        {
            let bpe = best_rate.entry(curstate.time_elapsed).or_default();
            if curstate.pressure_released > *bpe {
                *bpe = curstate.pressure_released;
            }
            if curstate.pressure_released
                < *best_rate
                    .get(&curstate.time_elapsed.saturating_sub(1))
                    .unwrap_or(&0)
            {
                continue;
            }
        }
        if curstate.time_elapsed <= 26 && curstate.pressure_released > best_pressure {
            best_pressure = curstate.pressure_released;
            best_state = Some(curstate.clone())
        }
        let actions = curstate.actions(valvemap);
        for (ma, ea) in actions {
            expstack.push(curstate.clone().apply_actions(valvemap, ma, ea));
        }
    }
    println!("Part 2 best pressure: {:?}", best_pressure);
    if let Some(best_state) = best_state {
        let history = best_state.history.reverse();
        let acts: Vec<&(Action<'_>, Action<'_>)> = history.iter().collect();
        println!("{:?}", acts);
    }

    Ok(())
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let input = io::read_to_string(io::stdin())?;
    let linere =
        regex::Regex::new(r"Valve (.+) has flow rate=(\d+); tunnels? leads? to valves? (.*)")?;

    let mut valvemap: HashMap<String, Valve> = HashMap::new();
    for line in input.lines() {
        let caps = linere
            .captures(line)
            .ok_or_else(|| eyre!("malformed line: {:?}", line))?;
        let src = caps[1].to_owned();
        let flowrate: u32 = caps[2].parse()?;
        let exits: Vec<String> = caps[3].split(", ").map(str::to_owned).collect();
        println!("{src} {flowrate} -> {exits:?}");
        valvemap.insert(src, Valve { flowrate, exits });
    }

    do_part1(&valvemap)?;
    do_part2(&valvemap)?;
    Ok(())
}
