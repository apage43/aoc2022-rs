use color_eyre::{
    eyre::{ContextCompat},
    Result,
};
use regex::Regex;

enum ParsingState {
    ReadingStack,
    ReadingMoves,
}

fn parse_crate_stacks(mut stacklines: Vec<String>) -> Result<Vec<Vec<char>>> {
    let mut stacks: Vec<Vec<char>> = Vec::new();
    let numline_re = Regex::new(r"\b(\d+)\b")?;
    let mut stackcols = Vec::new();
    for caps in numline_re.captures_iter(&stacklines.pop().context("stack number line pop")?) {
        let m = caps.get(1).context("stack number match")?;
        stackcols.push(m.start());
    }
    stacks.resize_with(stackcols.len(), Default::default);
    while let Some(crateline) = stacklines.pop() {
        for (idx, stackcol) in stackcols.iter().enumerate() {
            let cratech = crateline.get(*stackcol..).unwrap_or(" ").chars().next();
            match cratech {
                Some(' ') | None => {}
                Some(c) => stacks[idx].push(c),
            }
        }
    }
    Ok(stacks)
}

fn print_stacks(mut stacks: Vec<Vec<char>>) {
    let mut lines = Vec::new();
    let nstacks = stacks.len();
    let botline = (0..nstacks)
        .map(|i| format!(" {} ", i + 1))
        .collect::<Vec<_>>()
        .join(" ");
    lines.push(botline);
    for i in 0..nstacks {
        stacks[i].reverse()
    }
    loop {
        let mut ncrates = 0;
        let mut segments = Vec::new();
        for i in 0..nstacks {
            if let Some(cratech) = stacks[i].pop() {
                segments.push(format!("[{}]", cratech));
                ncrates += 1;
            } else {
                segments.push("   ".to_string())
            }
        }
        if ncrates > 0 {
            lines.push(segments.join(" "));
        } else {
            break;
        }
    }
    lines.reverse();
    eprintln!("{}", lines.join("\n"));
}

fn main() -> Result<()> {
    color_eyre::install()?;
    // use arg --part2 for part2
    let do_part2 = std::env::args().any(|i| i == "--part2");
    let mut stacklines = Vec::new();
    use ParsingState::*;
    let mut state = ReadingStack;
    let mut moves: Vec<(u32, u32, u32)> = Vec::new();
    for line in std::io::stdin().lines() {
        let line = line?;
        match state {
            ReadingStack => {
                if line.is_empty() {
                    state = ReadingMoves;
                    eprintln!("stacks read, reading moves");
                } else {
                    stacklines.push(line);
                }
            }
            ReadingMoves => {
                let (ncrates, from, to) = strp::try_scan!(line => "move {} from {} to {}")
                    .ok()
                    .context("reading move")?;
                moves.push((ncrates, from, to));
            }
        }
    }
    eprintln!("moves read, parsing stacks");
    let mut stacks = parse_crate_stacks(stacklines)?;
    eprintln!("executing moves");
    for (n, from, to) in moves {
        // eprintln!("move {} from {} to {}", n, from, to);
        if do_part2 {
            let fromheight = stacks[from as usize-1].len();
            let mut taken = stacks[from as usize-1].split_off(fromheight - n as usize);
            stacks[to as usize-1].append(&mut taken);
        } else {
            for _i in 0..n {
                let taken = stacks[from as usize - 1]
                    .pop()
                    .context("move from empty stack")?;
                stacks[to as usize - 1].push(taken);
            }
        }
    }
    eprintln!("End state:");
    print_stacks(stacks.clone());
    let msg = stacks
        .iter()
        .map(|stack| stack.last().unwrap_or(&' '))
        .collect::<String>();
    println!("Final message: {}", msg);
    Ok(())
}
