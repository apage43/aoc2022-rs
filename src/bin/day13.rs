use std::cmp::{Ord, Ordering};

use color_eyre::{
    eyre::{bail, ContextCompat},
    Result,
};

#[derive(Debug, PartialEq, Eq, Clone)]
enum Value {
    List(Vec<Value>),
    Num(i32),
}

impl Value {
    fn from_str(s: &str) -> Result<Value> {
        use Value::*;
        let mut stack: Vec<Vec<Value>> = vec![];
        let mut num = String::new();
        for ch in s.chars() {
            match ch {
                digit @ '0'..='9' => {
                    num.push(digit);
                }
                ',' => {
                    if !num.is_empty() {
                        stack
                            .last_mut()
                            .context("working list")?
                            .push(Num(num.parse()?));
                        num.clear();
                    }
                }
                '[' => {
                    stack.push(vec![]);
                }
                ']' => {
                    if !num.is_empty() {
                        stack
                            .last_mut()
                            .context("working list")?
                            .push(Num(num.parse()?));
                        num.clear();
                    }
                    let el = Value::List(stack.pop().context("mismatched ]")?);
                    if stack.is_empty() {
                        return Ok(el);
                    } else {
                        stack.last_mut().context("] push")?.push(el);
                    }
                }
                _ => (),
            }
        }
        bail!("Expected ]");
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Value::List(ll), Value::List(lr)) => ll.cmp(lr),
            (vr @ Value::List(_), rn @ Value::Num(_)) => vr.cmp(&Value::List(vec![rn.clone()])),
            (ln @ Value::Num(_), vl @ Value::List(_)) => Value::List(vec![ln.clone()]).cmp(vl),
            (Value::Num(nl), Value::Num(nr)) => nl.cmp(nr),
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let input = std::io::read_to_string(std::io::stdin())?;
    let mut pairs = vec![];
    for linepair in input.trim().split("\n\n") {
        let mut lit = linepair.lines();
        let a = Value::from_str(lit.next().context("pair_left")?)?;
        let b = Value::from_str(lit.next().context("pair_right")?)?;
        pairs.push((a, b));
    }
    let mut mismatch_indices = vec![];
    for (i, (a, b)) in pairs.iter().enumerate() {
        let cmp = a.cmp(b);
        println!("Pair {}: l {:?} r", i + 1, cmp);
        if cmp == Ordering::Less {
            mismatch_indices.push(i + 1);
        }
    }
    println!(
        "Mismatch index sum: {}",
        mismatch_indices.iter().sum::<usize>()
    );
    let mut all_packets: Vec<Value> = input
        .lines()
        .filter(|s| !s.is_empty())
        .map(|s| Ok(Value::from_str(s)?))
        .collect::<Result<_>>()?;
    let marker_a = Value::from_str("[[2]]")?;
    let marker_b = Value::from_str("[[6]]")?;
    all_packets.extend([marker_a.clone(), marker_b.clone()]);
    all_packets.sort();
    let mut aidx = None;
    let mut bidx = None;
    for (i, p) in all_packets.iter().enumerate() {
        if p == &marker_a {
            aidx = Some(i + 1);
        }
        if p == &marker_b {
            bidx = Some(i + 1);
        }
    }
    let key = aidx.context("a idx")? * bidx.context("b idx")?;
    println!("Decoder key: {}", key);
    Ok(())
}
