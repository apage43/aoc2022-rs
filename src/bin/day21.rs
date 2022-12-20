use color_eyre::Result;
use core::panic;
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
    Unify(MonkeyId, MonkeyId),
    Variable,
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
                    Add(r, l) => Some(results.get(r)? + results.get(l)?),
                    Sub(r, l) => Some(results.get(r)? - results.get(l)?),
                    Mul(r, l) => Some(results.get(r)? * results.get(l)?),
                    Div(r, l) => Some(results.get(r)? / results.get(l)?),
                    _ => todo!(),
                }
            })() {
                results.insert(mid, result);
            }
        }
    }
    let rootresult = results.get(&rootid);
    let humnid = mmap.get_or_insert("humn");
    println!("p1 root monkey result: {rootresult:?}");

    if let MonkeyRule::Add(l, r) = mrules[&rootid] {
        mrules.insert(rootid, MonkeyRule::Unify(l, r));
        mrules.insert(humnid, MonkeyRule::Variable);
    } else {
        panic!("root not +")
    };
    let mut results: HashMap<MonkeyId, Term> = Default::default();
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
                    Shout(v) => Some(Term::Const(num::Rational64::from(*v))),
                    Variable => Some(Term::Var),
                    Add(r, l) => Some(Term::Add(
                        Box::new(results.get(r)?.clone()),
                        Box::new(results.get(l)?.clone()),
                    )),
                    Sub(r, l) => Some(Term::Sub(
                        Box::new(results.get(r)?.clone()),
                        Box::new(results.get(l)?.clone()),
                    )),
                    Mul(r, l) => Some(Term::Mul(
                        Box::new(results.get(r)?.clone()),
                        Box::new(results.get(l)?.clone()),
                    )),
                    Div(r, l) => Some(Term::Div(
                        Box::new(results.get(r)?.clone()),
                        Box::new(results.get(l)?.clone()),
                    )),
                    Unify(r, l) => Some(Term::Equal(
                        Box::new(results.get(r)?.clone()),
                        Box::new(results.get(l)?.clone()),
                    )),
                }
            })() {
                results.insert(mid, result);
            }
        }
    }
    let rootresult = results.get(&rootid).unwrap().clone().full_collapse();
    println!("p2 root monkey result: {rootresult:?}");

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Term {
    Var,
    Const(num::Rational64),
    Add(Box<Term>, Box<Term>),
    Sub(Box<Term>, Box<Term>),
    Mul(Box<Term>, Box<Term>),
    Div(Box<Term>, Box<Term>),
    Equal(Box<Term>, Box<Term>),
}

impl Term {
    fn full_collapse(self) -> Term {
        let mut out = self;
        loop {
            let c = out.clone().collapse();
            if c != out {
                out = c;
            } else {
                break;
            }
        }
        out
    }
    fn const_cancel(self, c: num::Rational64) -> (Term, num::Rational64) {
        use Term::*;
        match self {
            Var | Const(_) => (self.clone(), c),
            Add(l, r) => match (*l, *r) {
                (Const(oc), o) | (o, Const(oc)) => (o, c - oc),
                _ => panic!("const-eliminating without a const side"),
            },
            Sub(l, r) => match (*l, *r) {
                (o, Const(oc)) => (o, c + oc),
                (Const(oc), o) => (o, oc - c),
                _ => panic!("const-eliminating without a const side"),
            },
            Mul(l, r) => match (*l, *r) {
                (Const(oc), o) | (o, Const(oc)) => (o, c / oc),
                _ => panic!("const-eliminating without a const side"),
            },
            Div(l, r) => match (*l, *r) {
                (o, Const(oc)) => (o, c * oc),
                (Const(oc), o) => (o, oc / c),
                _ => panic!("const-eliminating without a const side"),
            },
            _ => todo!(),
        }
    }
    fn collapse(self) -> Term {
        use Term::*;
        match self {
            Var | Const(_) => self.clone(),
            Add(l, r) => match (*l, *r) {
                (Const(lc), Const(rc)) => Const(lc + rc),
                (l, r) => Add(Box::new(l.collapse()), Box::new(r.collapse())),
            },
            Sub(l, r) => match (*l, *r) {
                (Const(lc), Const(rc)) => Const(lc - rc),
                (l, r) => Sub(Box::new(l.collapse()), Box::new(r.collapse())),
            },
            Mul(l, r) => match (*l, *r) {
                (Const(lc), Const(rc)) => Const(lc * rc),
                (l, r) => Mul(Box::new(l.collapse()), Box::new(r.collapse())),
            },
            Div(l, r) => match (*l, *r) {
                (Const(lc), Const(rc)) => Const(lc / rc),
                (l, r) => Div(Box::new(l.collapse()), Box::new(r.collapse())),
            },
            Equal(l, r) => match (*l, *r) {
                (Const(_), Const(_)) => unimplemented!(),
                (Const(c), r) | (r, Const(c)) => {
                    let (nr, nc) = r.full_collapse().const_cancel(c);
                    Equal(Box::new(Const(nc)), Box::new(nr))
                }
                (l, r) => Equal(Box::new(l.collapse()), Box::new(r.collapse())),
            },
        }
    }
}
