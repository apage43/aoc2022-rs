use color_eyre::{
    eyre::{bail, ContextCompat},
    Result,
};

#[derive(Clone, Copy)]
enum RPSPlay {
    Rock,
    Paper,
    Scissors,
}

enum RPSOutcome {
    Win,
    Lose,
    Draw,
}

impl RPSOutcome {
    fn decode(inp: &str) -> Result<RPSOutcome> {
        use RPSOutcome::*;
        match inp {
            "X" => Ok(Lose),
            "Y" => Ok(Draw),
            "Z" => Ok(Win),
            _ => bail!("Unknown play"),
        }
    }

    fn vs(self, opponent: RPSPlay) -> RPSPlay {
        use RPSOutcome::*;
        use RPSPlay::*;
        match (self, opponent) {
            (Win, Rock) => Paper,
            (Lose, Rock) => Scissors,
            (Win, Paper) => Scissors,
            (Lose, Paper) => Rock,
            (Win, Scissors) => Rock,
            (Lose, Scissors) => Paper,
            (Draw, x) => x,
        }
    }
}

impl RPSPlay {
    fn decode_part1(inp: &str) -> Result<RPSPlay> {
        use RPSPlay::*;
        match inp {
            "A" | "X" => Ok(Rock),
            "B" | "Y" => Ok(Paper),
            "C" | "Z" => Ok(Scissors),
            _ => bail!("Unknown play"),
        }
    }
}

struct RPSRound {
    opponent: RPSPlay,
    mine: RPSPlay,
}

impl RPSRound {
    fn score(self) -> u32 {
        use RPSPlay::*;
        let base_score = match self.mine {
            Rock => 1,
            Paper => 2,
            Scissors => 3,
        };
        let win_score = match (self.mine, self.opponent) {
            (Rock, Scissors) => 6,
            (Rock, Paper) => 0,
            (Paper, Rock) => 6,
            (Paper, Scissors) => 0,
            (Scissors, Paper) => 6,
            (Scissors, Rock) => 0,
            (_, _) => 3,
        };
        base_score + win_score
    }

    fn decode_part1(inp: &str) -> Result<RPSRound> {
        let mut splits = inp.split_whitespace();
        let opponent = RPSPlay::decode_part1(splits.next().context("opponent move")?)?;
        let mine = RPSPlay::decode_part1(splits.next().context("my move")?)?;
        Ok(RPSRound { opponent, mine })
    }

    fn decode_part2(inp: &str) -> Result<RPSRound> {
        let mut splits = inp.split_whitespace();
        let opponent = RPSPlay::decode_part1(splits.next().context("opponent move")?)?;
        let outcome = RPSOutcome::decode(splits.next().context("intended outcome")?)?;
        let mine = outcome.vs(opponent);
        Ok(RPSRound { opponent, mine })
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut part1_score = 0;
    let mut part2_score = 0;

    for line in std::io::stdin().lines() {
        let line = line?;
        let roundp1 = RPSRound::decode_part1(line.as_str())?;
        let roundp2 = RPSRound::decode_part2(line.as_str())?;
        part1_score += roundp1.score();
        part2_score += roundp2.score();
    }
    println!("part 1 score {}", part1_score);
    println!("part 2 score {}", part2_score);
    Ok(())
}
