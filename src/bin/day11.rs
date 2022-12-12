use std::collections::VecDeque;
use std::io::Write;

use color_eyre::{
    eyre::{bail, Context, ContextCompat},
    Result,
};
use derive_builder::Builder;

#[derive(Debug, Clone)]
enum Operand {
    Old,
    Literal(i64),
}

#[derive(Debug, Clone)]
enum Expr {
    Sum(Operand),
    Mul(Operand),
}

#[derive(Debug, Builder)]
struct Monkey {
    items: VecDeque<i64>,
    op: Expr,
    divisor: i64,
    test_target_true: usize,
    test_target_false: usize,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let do_part2 = std::env::args().any(|x| x == "--part2");
    let mut current_monkey: Option<MonkeyBuilder> = None;
    let mut monkeys = Vec::new();
    for line in std::io::stdin().lines() {
        let line = line?;
        let line = line.trim();
        if current_monkey.is_none() {
            current_monkey = Some(MonkeyBuilder::default());
        }
        let builder = current_monkey.as_mut().context("current monkey")?;
        if let Some(rest) = line.strip_prefix("Starting items: ") {
            builder.items(
                rest.split(", ")
                    .map(|x| x.parse().wrap_err("parse items"))
                    .collect::<Result<_>>()?,
            );
        }
        if let Some(rest) = line.strip_prefix("Operation: new = old ") {
            let parts = rest.split_whitespace().collect::<Vec<_>>();
            let op_right = match parts[1] {
                "old" => Operand::Old,
                x => Operand::Literal(x.parse()?),
            };
            builder.op(match parts[0] {
                "*" => Expr::Mul(op_right),
                "+" => Expr::Sum(op_right),
                x => bail!("unknown operator {}", x),
            });
        }
        if let Some(rest) = line.strip_prefix("Test: divisible by ") {
            builder.divisor(rest.parse()?);
        }
        if let Some(rest) = line.strip_prefix("If true: throw to monkey ") {
            builder.test_target_true(rest.parse()?);
        }
        if let Some(rest) = line.strip_prefix("If false: throw to monkey ") {
            builder.test_target_false(rest.parse()?);
            if let Some(finished) = current_monkey.take() {
                monkeys.push(finished.build()?);
            }
        }
    }
    println!("Initial monkeys:");
    for m in &monkeys {
        println!("  {:?}", m);
    }
    let dprod: i64 = monkeys.iter().map(|m| m.divisor).product();
    println!("Product of all test divisors: {}", dprod);
    let mut monkey_activity = vec![0; monkeys.len()];
    for round in 0..(if do_part2 { 10000 } else { 20 }) {
        print!("round {}...\r", round + 1);
        std::io::stdout().flush()?;
        do_round(&mut monkeys, &mut monkey_activity, do_part2, dprod);
    }
    println!();
    println!("Items after rounds:");
    print_items(&monkeys);
    println!("Monkey activity report: {:?}", monkey_activity);
    monkey_activity.sort();
    let monkey_business: u64 = monkey_activity[monkey_activity.len() - 2..]
        .iter()
        .product();
    println!("Monkey business: {}", monkey_business);
    Ok(())
}

fn print_items(monkeys: &Vec<Monkey>) {
    for idx in 0..monkeys.len() {
        println!("Monkey {}: {:?}", idx, monkeys[idx].items);
    }
}
fn do_round(monkeys: &mut Vec<Monkey>, activity: &mut Vec<u64>, part2: bool, dprod: i64) {
    for idx in 0..monkeys.len() {
        while let Some(worry) = monkeys[idx].items.pop_front() {
            activity[idx] += 1;
            //println!("Monkey inspects item with worry level {}", worry);
            let prev_worry = worry;
            let worry = match &monkeys[idx].op {
                Expr::Sum(Operand::Literal(a)) => prev_worry + a,
                Expr::Mul(Operand::Literal(a)) => prev_worry * a,
                Expr::Sum(Operand::Old) => prev_worry + prev_worry,
                Expr::Mul(Operand::Old) => prev_worry * prev_worry,
            };
            // boredom / mod
            let worry = if !part2 { worry / 3 } else { worry % dprod };
            // throw
            let target = if worry % monkeys[idx].divisor == 0 {
                monkeys[idx].test_target_true
            } else {
                monkeys[idx].test_target_false
            };
            monkeys[target].items.push_back(worry);
        }
    }
}
