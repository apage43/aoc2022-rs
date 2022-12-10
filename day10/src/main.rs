use std::collections::VecDeque;

use color_eyre::{
    eyre::{bail, ContextCompat},
    Result,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Instruction {
    Noop,
    Addx(i32),
}

impl Instruction {
    fn from_str(text: &str) -> Result<Instruction> {
        let mut splits = text.split_whitespace();
        let op = splits.next().context("reading opcode name")?;
        match op {
            "noop" => Ok(Instruction::Noop),
            "addx" => {
                let param = splits.next().context("reading addx param")?;
                let param: i32 = param.parse()?;
                Ok(Instruction::Addx(param))
            }
            _ => bail!("unknown opcode"),
        }
    }
}

#[derive(Debug, Default)]
struct Machine {
    xreg: i32,
    cycle_counter: u64,
    iqueue: VecDeque<Instruction>,
    delayed: Option<(u64, Instruction)>,
}

impl Machine {
    fn new() -> Machine {
        Machine {
            xreg: 1,
            cycle_counter: 1,
            ..Default::default()
        }
    }
    fn enqueue_instruction(&mut self, instr: Instruction) {
        self.iqueue.push_back(instr);
    }
    fn begin_instruction(&mut self, instr: Instruction) {
        match instr {
            Instruction::Noop => {
                self.delayed = Some((self.cycle_counter, instr));
            }
            Instruction::Addx(_) => {
                self.delayed = Some((self.cycle_counter + 1, instr));
            }
        }
    }
    fn tick(&mut self) -> Result<()> {
        if self.delayed.is_none() {
            if let Some(instr) = self.iqueue.pop_front() {
                self.begin_instruction(instr);
            }
        }
        self.cycle_counter += 1;
        if let Some((at_cycle, retire)) = self.delayed {
            if at_cycle < self.cycle_counter {
                match retire {
                    Instruction::Noop => {}
                    Instruction::Addx(p) => {
                        self.xreg += p;
                    }
                }
                self.delayed = None;
            }
        }
        Ok(())
    }
    fn is_done(&self) -> bool {
        self.delayed.is_none() && self.iqueue.is_empty()
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut machine = Machine::new();
    for line in std::io::stdin().lines() {
        let line = line?;
        let instr = Instruction::from_str(&line)?;
        machine.enqueue_instruction(instr);
    }
    let mut sigsamples = Vec::new();
    let mut line = String::new();
    while !machine.is_done() {
        if machine.cycle_counter == 20 || (machine.cycle_counter + 20) % 40 == 0 {
            let signal = machine.cycle_counter as i32 * machine.xreg;
            sigsamples.push(signal);
        }
        let ppos: i32 = (machine.cycle_counter as i32 - 1) % 40;
        let lit = ppos >= (machine.xreg - 1) && ppos <= (machine.xreg + 1);
        if lit {
            line.push('#');
        } else {
            line.push('.')
        }
        if line.len() == 40 {
            println!("> {}  @{}", line, machine.cycle_counter);
            line.clear();
        }
        machine.tick()?;
    }
    println!("final machine state: {:?}", machine);
    println!(
        "signal samples, {:?}, total: {}",
        sigsamples,
        sigsamples.iter().sum::<i32>()
    );
    Ok(())
}
